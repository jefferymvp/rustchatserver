use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub mod args;
pub mod models;
pub mod routes;
pub mod server;
pub mod storage;
pub mod ws;

use crate::args::ARGS;
use crate::routes::{auth, doc, notice};
use crate::server::ChatServer;
use crate::storage::Storage;
use crate::ws::ws_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    // 确保数据存储目录与上传目录存在
    let _ = std::fs::create_dir_all(&ARGS.data_dir);
    let _ = std::fs::create_dir_all("public/doc");

    let db_path = std::path::Path::new(&ARGS.data_dir).join("chat_history.json");
    let storage = Storage::new(&db_path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("初始化持久化存储引擎失败: {}", e),
        )
    })?;

    let chat_server = ChatServer::new(storage);
    let app_data = web::Data::new(chat_server);

    println!("============================================================");
    println!("✔ Chatroom 聊天室服务 (Rust 版) 已成功启动！");
    println!("ℹ 访问地址: http://{}:{}", ARGS.bind, ARGS.port);
    println!("ℹ 静态资源目录: {}", ARGS.public_dir);
    println!("ℹ 历史记录持久化文件: {}", db_path.display());
    println!("============================================================");

    let max_payload_bytes = ARGS.max_upload_size_mb * 1024 * 1024;

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .app_data(web::PayloadConfig::new(max_payload_bytes))
            .wrap(middleware::Logger::default())
            .service(auth::index)
            .service(auth::signin_page)
            .service(auth::signin_post)
            .service(auth::signup_page)
            .service(doc::serve_doc)
            .service(notice::notice_handler)
            .route("/ws", web::get().to(ws_handler))
            .service(Files::new("/", "./public").prefer_utf8(true))
    })

    .bind((ARGS.bind, ARGS.port))?
    .run()
    .await
}
