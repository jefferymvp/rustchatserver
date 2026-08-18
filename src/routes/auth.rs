use actix_files::NamedFile;
use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct SigninForm {
    pub username: Vec<String>,
}

#[get("/")]
pub async fn index(req: HttpRequest) -> Result<HttpResponse> {
    let mut is_login = false;
    if let Some(cookie) = req.cookie("user") {
        if !cookie.value().trim().is_empty() {
            is_login = true;
        }
    }

    if !is_login {
        return Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, "/signin"))
            .finish());
    }

    let path: PathBuf = "views/index.html".into();
    Ok(NamedFile::open(path)?.into_response(&req))
}

#[get("/signin")]
pub async fn signin_page(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "views/signin.html".into();
    Ok(NamedFile::open(path)?)
}

#[post("/signin")]
pub async fn signin_post(form: web::Form<SigninForm>) -> impl Responder {
    let username = form.username.first().cloned().unwrap_or_default();
    let cookie = Cookie::build("user", username).path("/").finish();

    HttpResponse::Found()
        .cookie(cookie)
        .insert_header((header::LOCATION, "/"))
        .finish()
}

#[get("/signup")]
pub async fn signup_page(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "views/signup.html".into();
    Ok(NamedFile::open(path)?)
}

