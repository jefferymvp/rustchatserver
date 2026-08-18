use crate::models::{FileUploadPayload, SayMsg, SayPayload, WsEnvelope};
use crate::server::ChatServer;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_ws::Message;
use chrono::Local;
use futures_util::StreamExt;
use std::path::Path;
use tokio::sync::mpsc;

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<ChatServer>,
) -> Result<HttpResponse, Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
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

    // 接收客户端上行的 WebSocket 消息
    actix_web::rt::spawn(async move {
        let mut current_user: Option<String> = None;

        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => {
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
                                if let Ok(payload) =
                                    serde_json::from_value::<SayPayload>(envelope.data)
                                {
                                    let say_msg = SayMsg {
                                        time: Local::now().timestamp_millis(),
                                        data: payload,
                                        notice: true,
                                    };
                                    server_clone.broadcast_say(say_msg).await;
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
                                if let Ok(upload) =
                                    serde_json::from_value::<FileUploadPayload>(envelope.data)
                                {
                                    let doc_dir = Path::new("public/doc");
                                    let _ = std::fs::create_dir_all(doc_dir);
                                    let file_path = doc_dir.join(&upload.file_name);
                                    let _ = std::fs::write(&file_path, &upload.file_buffer);
                                    log::info!("文件已上传保存: {:?}", file_path);
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
                Message::Close(_) => {
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
