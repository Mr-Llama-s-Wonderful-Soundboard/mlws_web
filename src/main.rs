use actix_web::{web, App, HttpResponse, HttpServer, Responder, body::Body};

use std::io::Read;
use std::fs::File;
use tera::Context;

use mlws_lib::config::{Config, SoundConfig};
use mlws_lib;

mod template;
mod ws;

pub type ServerData = (Config, SoundConfig, (mlws_lib::SoundSender, mlws_lib::SoundReceiver, mlws_lib::SoundLoop));

async fn index(data: web::Data<ServerData>) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("repos", &data.1.json_sounds());
    HttpResponse::Ok().body(template::render_context(
        "index.html",
        &ctx
    ))
}

async fn css_handler(p: web::Path<(String,)>) -> impl Responder {
   
    HttpResponse::Ok().body(template::load(&format!("css/{}", p.0)))
}

async fn sound_img_handler(data: web::Data<ServerData>, p: web::Path<(String,String)>) -> impl Responder {
    match data.1.get(&p.0, &p.1) {
        Some(sound) => {
            let mut buf = Vec::new();
            File::open(sound.img.clone().unwrap()).expect("Error opening file").read_to_end(&mut buf).expect("Error reading file");
            HttpResponse::Ok().body(Body::from_slice(&buf))
        },
        None => HttpResponse::NotFound().finish()
    }
}

async fn sound_play_handler(data: web::Data<ServerData>, p: web::Path<(String,String)>) -> impl Responder {
    match data.1.get(&p.0, &p.1) {
        Some(sound) => {
            data.2.0.send(mlws_lib::sound::Message::PlaySound(sound.clone(), mlws_lib::sound::SoundDevices::Both)).expect("Error sending message");
            HttpResponse::Ok()
        },
        None => HttpResponse::NotFound()
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Setting up mlws");
    let (sound_sender, sound_receiver, mut soundloop) = mlws_lib::setup();
        soundloop.run().expect("Error starting soundloop");
    println!("Loading config");
    let mut config = Config::load();
    println!("Loading sounds");
    let sounds = SoundConfig::load(&mut config).await;
    config.save();
    let mut keybinds = mlws_lib::keybind::KeyBindings::new(sound_sender.clone(), config.clone(), |(repo, name)|{
        mlws_lib::sound::Message::PlaySound(sounds.get(&repo, &name).unwrap().clone(), mlws_lib::sound::SoundDevices::Both)
    });

    // sound_sender.send(mlws_lib::sound::Message::PlaySound(SoundConfig::load().get(&String::from("Our anthem")).unwrap().clone(), mlws_lib::sound::SoundDevices::Both)).expect("Error sending message");
    HttpServer::new(move || {
        let sender = sound_sender.clone();
        let receiver = sound_receiver.clone();
        let soundloop = soundloop.clone();
        let config = config.clone();
        let sound_config = sounds.clone();
        App::new()
            .data((config, sound_config, (sender, receiver, soundloop)))
            .route("/", web::get().to(index))
            .route("/ws", web::get().to(ws::ws))
            .route("/css/{file}", web::get().to(css_handler))
            // .route("/img/{name}", web::get().to(img_handler))
            .route("/sound/{repo}/{name}/img", web::get().to(sound_img_handler))
            .route("/sound/{repo}/{name}/play", web::get().to(sound_play_handler))
    })
    .bind("127.0.0.1:8088")?.bind("192.168.1.66:8088")?
    .run()
    .await
}
