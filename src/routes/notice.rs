use crate::server::ChatServer;
use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct NoticeQuery {
    pub msg: Option<String>,
}

#[get("/notice")]
pub async fn notice_handler(
    query: web::Query<NoticeQuery>,
    server: web::Data<ChatServer>,
) -> impl Responder {
    let msg_text = query.msg.clone().unwrap_or_default();
    server.broadcast_notice(msg_text).await;
    HttpResponse::Ok().body("ok")
}
