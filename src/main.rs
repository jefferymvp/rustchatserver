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
pub mod ws;

use crate::args::ARGS;
use crate::routes::{auth, notice};
use crate::server::ChatServer;
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

    let chat_server = ChatServer::new();
    let app_data = web::Data::new(chat_server);

    // 确保上传目录存在
    let _ = std::fs::create_dir_all("public/doc");

    println!("============================================================");
    println!("✔ Chatroom 聊天室服务 (Rust 版) 已成功启动！");
    println!("ℹ 访问地址: http://{}:{}", ARGS.bind, ARGS.port);
    println!("ℹ 静态资源目录: {}", ARGS.public_dir);
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
            .service(notice::notice_handler)
            .route("/ws", web::get().to(ws_handler))
            .service(Files::new("/", "./public").prefer_utf8(true))
    })

    .bind((ARGS.bind, ARGS.port))?
    .run()
    .await
}
