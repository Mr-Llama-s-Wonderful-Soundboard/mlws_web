use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use serde::Serialize;

mod template;

#[derive(Debug, Serialize)]
struct Index<'a> {
    sounds: &'a [Sound<'a>],
}

#[derive(Debug, Serialize)]
struct Sound<'a> {
    name: &'a str,
}

impl<'a> From<&'a str> for Sound<'a> {
    fn from(s: &'a str) -> Self {
        Self { name: s }
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body(template::render(
        "index.html",
        Index {
            sounds: &[Sound::from("Hola"), Sound::from("Adios"), Sound::from("Adios"), Sound::from("Adios"), Sound::from("Adios"), Sound::from("Hola"), Sound::from("Adios"), Sound::from("Adios"), Sound::from("Adios"), Sound::from("Adios")],
        },
    ))
}

async fn css_handler(p: web::Path<(String,)>) -> impl Responder {
   
    HttpResponse::Ok().body(template::load(&format!("css/{}", p.0)))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/css/{file}", web::get().to(css_handler))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
