use actix_files::NamedFile;
use actix_web::cookie::Cookie;
use actix_web::http::header;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct SigninForm {
    #[serde(deserialize_with = "deserialize_string_or_vec")]
    pub username: String,
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct StringOrVec;

    impl<'de> serde::de::Visitor<'de> for StringOrVec {
        type Value = String;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(value.to_string())
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            if let Some(first) = seq.next_element::<String>()? {
                Ok(first)
            } else {
                Ok(String::new())
            }
        }
    }

    deserializer.deserialize_any(StringOrVec)
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
    let username = form.username.trim().to_string();
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

