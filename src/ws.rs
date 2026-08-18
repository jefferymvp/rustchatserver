use crate::models::{FileUploadPayload, SayMsg, SayPayload, WsEnvelope};
use crate::server::ChatServer;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::AggregatedMessage;
use chrono::Local;
use futures_util::StreamExt;
use std::path::Path;
use tokio::sync::mpsc;

/// 构造一条只发给上传者的 upload_ack 通知（仅触发浏览器 Notification，不写入聊天记录）
fn make_upload_ack(msg: &str) -> String {
    serde_json::json!({
        "type": "upload_ack",
        "data": { "msg": msg }
    })
    .to_string()
}

/// 清理并安全提取文件名（完全支持中文字符与跨平台斜杠）
fn sanitize_filename(name: &str) -> String {
    let normalized = name.replace('\\', "/");
    let base = normalized.rsplit('/').next().unwrap_or(name);
    // 过滤 Windows 文件系统非法的 ASCII 字符，不过滤中文字符（《》等是合法 Unicode）
    let filtered: String = base
        .chars()
        .filter(|&c| c != ':' && c != '*' && c != '?' && c != '"' && c != '<' && c != '>' && c != '|' && c != '/' && c != '\\')
        .collect();
    if filtered.trim().is_empty() {
        format!("file_{}", Local::now().timestamp())
    } else {
        filtered.trim().to_string()
    }
}

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<ChatServer>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, msg_stream) = actix_ws::handle(&req, stream)?;

    // 关键修复：aggregate_continuations 会将浏览器自动分片的大消息重新拼装为完整消息
    // max_frame_size: 单帧上限 50MB；max_continuation_size: 合并后消息总大小上限 100MB
    let mut msg_stream = msg_stream
        .max_frame_size(50 * 1024 * 1024)
        .aggregate_continuations()
        .max_continuation_size(100 * 1024 * 1024);

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let server_clone = server.get_ref().clone();

    // 接收服务端广播并向客户端 WebSocket 推送
    actix_web::rt::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if session.text(msg).await.is_err() {
                break;
            }
        }
    });

    // 接收客户端上行的 WebSocket 消息（已自动重组分片消息）
    actix_web::rt::spawn(async move {
        let mut current_user: Option<String> = None;

        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                AggregatedMessage::Text(text) => {
                    if let Ok(envelope) = serde_json::from_str::<WsEnvelope>(&text) {
                        match envelope.event_type.as_str() {
                            "online" => {
                                if let Some(username) = envelope
                                    .data
                                    .get("user")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string())
                                {
                                    current_user = Some(username.clone());
                                    server_clone.connect_user(username, tx.clone()).await;
                                }
                            }
                            "say" => {
                                match serde_json::from_value::<SayPayload>(envelope.data.clone()) {
                                    Ok(payload) => {
                                        let say_msg = SayMsg {
                                            time: Local::now().timestamp_millis(),
                                            data: payload,
                                            notice: true,
                                        };
                                        server_clone.broadcast_say(say_msg).await;
                                    }
                                    Err(e) => {
                                        log::warn!("解析 say 消息失败: {}, 原始数据: {:?}", e, envelope.data);
                                    }
                                }
                            }
                            "image" => {
                                server_clone.broadcast_image(envelope.data).await;
                            }
                            "notice" => {
                                if let Some(msg_text) =
                                    envelope.data.as_str().map(|s| s.to_string())
                                {
                                    server_clone.broadcast_notice(msg_text).await;
                                }
                            }
                            "clear_history" => {
                                server_clone.clear_history().await;
                            }
                            "fileUpload" => {
                                log::info!("[fileUpload] 收到上传请求, 完整消息长度: {} 字节", text.len());

                                match serde_json::from_value::<FileUploadPayload>(envelope.data.clone()) {
                                    Ok(upload) => {
                                        let safe_name = sanitize_filename(&upload.file_name);
                                        log::info!(
                                            "[fileUpload] 原始文件名: {}, 安全文件名: {}, Base64 数据长度: {}",
                                            upload.file_name, safe_name, upload.file_buffer.len()
                                        );

                                        let doc_dir = Path::new("public/doc");
                                        let _ = std::fs::create_dir_all(doc_dir);
                                        let file_path = doc_dir.join(&safe_name);

                                        use base64::Engine;
                                        // 去除 Data URL 前缀（如 data:application/pdf;base64,）
                                        let raw_b64 = if let Some(idx) = upload.file_buffer.find(',') {
                                            upload.file_buffer[idx + 1..].to_string()
                                        } else {
                                            upload.file_buffer.clone()
                                        };

                                        if raw_b64.trim().is_empty() {
                                            let msg = format!("❌ 文件 {} 上传失败：Base64 内容为空", safe_name);
                                            log::error!("{}", msg);
                                            let _ = tx.send(make_upload_ack(&msg));
                                        } else {
                                            match base64::engine::general_purpose::STANDARD.decode(raw_b64.trim()) {
                                                Ok(bytes) => {
                                                    match std::fs::write(&file_path, &bytes) {
                                                        Ok(_) => {
                                                            let msg = format!(
                                                                "✅ 文件 {} 上传成功（{} 字节）",
                                                                safe_name, bytes.len()
                                                            );
                                                            println!(
                                                                "✔ [文件上传成功] 已保存至: {:?} ({} 字节)",
                                                                file_path, bytes.len()
                                                            );
                                                            log::info!("文件已成功保存: {:?}", file_path);
                                                            let _ = tx.send(make_upload_ack(&msg));
                                                        }
                                                        Err(e) => {
                                                            let msg = format!("❌ 文件 {} 写入磁盘失败：{}", safe_name, e);
                                                            log::error!("写入上传文件失败: {:?}, 错误: {}", file_path, e);
                                                            let _ = tx.send(make_upload_ack(&msg));
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    let msg = format!("❌ 文件 {} Base64 解码失败：{}", safe_name, e);
                                                    log::warn!("Base64 解码失败: {}, 错误: {:?}", safe_name, e);
                                                    let _ = tx.send(make_upload_ack(&msg));
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let msg = format!("❌ 文件上传数据格式错误：{}", e);
                                        let preview: String = text.chars().take(200).collect();
                                        log::warn!("[fileUpload] 解析失败: {:?}, 消息前 200 字符: {:?}", e, preview);
                                        let _ = tx.send(make_upload_ack(&msg));
                                    }
                                }
                            }
                            "offline" => {
                                if let Some(ref username) = current_user {
                                    server_clone.disconnect_user(username).await;
                                }
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                AggregatedMessage::Close(_) => {
                    break;
                }
                _ => {}
            }
        }

        if let Some(ref username) = current_user {
            server_clone.disconnect_user(username).await;
        }
    });

    Ok(response)
}
