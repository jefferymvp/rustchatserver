use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use std::path::PathBuf;

/// 专用的 URL percent-decode 解码器（零额外依赖，完全支持 UTF-8 中文字符）
fn decode_percent(input: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = input.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let h1 = chars.next();
            let h2 = chars.next();
            if let (Some(h1), Some(h2)) = (h1, h2) {
                if let Ok(val) = u8::from_str_radix(std::str::from_utf8(&[h1, h2]).unwrap_or(""), 16) {
                    bytes.push(val);
                    continue;
                }
            }
        }
        bytes.push(b);
    }
    String::from_utf8(bytes).unwrap_or_else(|_| input.to_string())
}

/// 专用的 /doc/{filename:.*} 文件下载与预览路由，完美兼容中文字符与 URL 编码
#[get("/doc/{filename:.*}")]
pub async fn serve_doc(req: HttpRequest, path: web::Path<String>) -> impl Responder {
    let raw_name = path.into_inner();
    let decoded_name = decode_percent(&raw_name);

    // 安全过滤，防止路径穿越攻击（..）
    let safe_name = decoded_name.replace("..", "").replace('\\', "/");
    let safe_filename = safe_name.rsplit('/').next().unwrap_or(&safe_name);

    let doc_path = PathBuf::from("public/doc").join(safe_filename);

    if doc_path.exists() && doc_path.is_file() {
        match NamedFile::open_async(&doc_path).await {
            Ok(named_file) => named_file.into_response(&req),
            Err(e) => {
                log::error!("打开文件失败: {:?}, 错误: {:?}", doc_path, e);
                HttpResponse::InternalServerError()
                    .content_type("text/plain; charset=utf-8")
                    .body("无法读取请求的文件")
            }
        }
    } else {
        log::warn!("请求的文件不存在: {:?} (原始请求: {})", doc_path, raw_name);
        HttpResponse::NotFound()
            .content_type("text/plain; charset=utf-8")
            .body(format!("找不到文件: {}", safe_filename))
    }
}
