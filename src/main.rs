use actix_web::{web, App, HttpResponse, HttpServer, Responder};

use serde::Serialize;

mod template;

#[derive(Debug, Serialize)]
struct Index<'a> {
    user: &'a str
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body(template::render("index.html", Index {user: "Hi"}))
}

async fn index2() -> impl Responder {
    HttpResponse::Ok().body("Hello world again!")
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/again", web::get().to(index2))
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
