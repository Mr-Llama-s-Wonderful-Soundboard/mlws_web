// use actix_web::{web, App, HttpResponse, HttpServer, Responder, body::Body};

use std::fs::File;
use std::io::Read;
use tera::Context;

use mlws_lib;
use mlws_lib::config::{Config, SoundConfig};

use mime_guess;
// use serde_json;
use tokio;
// use tokio::prelude::*;
use warp;
use warp::{reply::Reply, Filter};

use std::sync::Arc;
use std::sync::RwLock;

use urldecode as url;

mod template;
mod ws;

pub type ServerData = Arc<RwLock<(
    Config,
    SoundConfig,
    (
        mlws_lib::SoundSender,
        mlws_lib::SoundReceiver,
        mlws_lib::SoundLoop,
    ),
)>>;

// async fn index(data: ServerData) -> impl Fn() -> String {
//     ||{
//         let mut ctx = Context::new();
//         ctx.insert("repos", &data.1.json_sounds());
//         template::render_context(
//             "index.html",
//             &ctx
//         )
//     }
// }

// async fn settings(data: ServerData) -> impl Fn() -> String {
//     ||{
//         let mut ctx = Context::new();
//         ctx.insert("config", &data.0);
//         ctx.insert("repos", &data.1.json_sounds());
//         template::render_context(
//             "settings.html",
//             &ctx
//         )
//     }
// }

// async fn css_handler(p: String) -> String {
//     template::load(&format!("css/{}", p))
// }

// async fn sound_img_handler(data: ServerData) -> impl Fn(String, String) -> String {
//     |repo, name |{
//         match data.1.get(&repo, &name) {
//             Some(sound) => {
//                 let mut buf = Vec::new();
//                 File::open(sound.img.clone().unwrap()).expect("Error opening file").read_to_end(&mut buf).expect("Error reading file");
//                 warp::reply::Response::new(buf)
//             },
//             None => 404
//         }
//     }
// }

// async fn sound_play_handler(data: web::Data<ServerData>, p: web::Path<(String,String)>) -> impl Responder {
//     match data.1.get(&p.0, &p.1) {
//         Some(sound) => {
//             data.2.0.send(mlws_lib::sound::Message::PlaySound(sound.clone(), mlws_lib::sound::SoundDevices::Both)).expect("Error sending message");
//             HttpResponse::Ok()
//         },
//         None => HttpResponse::NotFound()
//     }
// }

#[tokio::main]
async fn main() {
    println!("Setting up mlws");
    let (sound_sender, sound_receiver, mut soundloop) = mlws_lib::setup();
    soundloop.run().expect("Error starting soundloop");
    println!("Loading config");
    let mut config = Config::load();
    println!("Loading sounds");
    let sounds = SoundConfig::load(&mut config).await;
    config.save();
    let mut keybinds = mlws_lib::keybind::KeyBindings::new(
        sound_sender.clone(),
        config.clone(),
        |(repo, name)| {
            mlws_lib::sound::Message::PlaySound(
                sounds.get(&repo, &name).unwrap().clone(),
                mlws_lib::sound::SoundDevices::Both,
            )
        },
    );

    let data = (config, sounds, (sound_sender, sound_receiver, soundloop));

    let data_idx = data.clone();
    let index = warp::path::end().map(move || {
        let mut ctx = Context::new();
        ctx.insert("repos", &data_idx.1.json_sounds());
        warp::reply::html(template::render_context("index.html", &ctx))
    });

    let data_sett = data.clone();
    let settings = warp::path!("settings").map(move || {
        let mut ctx = Context::new();
        ctx.insert("config", &data_sett.0);
        ctx.insert("repos", &data_sett.1.json_sounds());
        warp::reply::html(template::render_context("settings.html", &ctx))
    });

    
    let css = warp::path!("css" / String).map(|p| warp::reply::with_header(template::load(&format!("css/{}", url::decode(p))), "Content-Type", "text/css"));
    
    let data_img = data.clone();
    let sound_img = warp::path!("sound" / String / String / "img").map(move |repo, name| {
        let repo = url::decode(repo);
        let name = url::decode(name);
        println!("REPO: {}; SOUND: {}", repo, name);
        match data_img.1.get(&repo, &name) {
            Some(sound) => {
                let mut buf = Vec::new();
                File::open(sound.img.clone().unwrap())
                    .expect("Error opening file")
                    .read_to_end(&mut buf)
                    .expect("Error reading file");
                warp::reply::with_header(
                    warp::reply::Response::new(buf.into()),
                    "Content-Type",
                    mime_guess::from_path(sound.img.clone().unwrap())
                        .first_or_text_plain()
                        .as_ref(),
                )
                .into_response()
            }
            None => warp::http::StatusCode::NOT_FOUND.into_response(),
        }
    });
    
    let ws = warp::path("ws").and(warp::ws()).map(
        move |ws: warp::ws::Ws| {
            let data = Arc::new(RwLock::new(data.clone()));
            let res = ws.on_upgrade(move |ws| ws::ws(ws, data));
            res
        }
    );

    let route = index.or(settings).or(css).or(sound_img).or(ws);
    // std::thread::spawn(move || {
    //     let mut rt = tokio::runtime::Runtime::new().unwrap();

    //     rt.block_on(async move {
    //             warp::serve(route_clone).run("192.168.1.66:8088".parse::<std::net::SocketAddr>().unwrap())
    //         })
    //     });

    warp::serve(route)
        .run("192.168.1.66:8088".parse::<std::net::SocketAddr>().unwrap())
        .await;

    // sound_sender.send(mlws_lib::sound::Message::PlaySound(SoundConfig::load().get(&String::from("Our anthem")).unwrap().clone(), mlws_lib::sound::SoundDevices::Both)).expect("Error sending message");
    // HttpServer::new(move || {
    //     let sender = sound_sender.clone();
    //     let receiver = sound_receiver.clone();
    //     let soundloop = soundloop.clone();
    //     let config = config.clone();
    //     let sound_config = sounds.clone();
    //     App::new()
    //         .data((config, sound_config, (sender, receiver, soundloop)))
    //         .route("/", web::get().to(index))
    //         .route("/settings", web::get().to(settings))
    //         .route("/ws", web::get().to(ws::ws))
    //         .route("/css/{file}", web::get().to(css_handler))
    //         // .route("/img/{name}", web::get().to(img_handler))
    //         .route("/sound/{repo}/{name}/img", web::get().to(sound_img_handler))
    //         .route("/sound/{repo}/{name}/play", web::get().to(sound_play_handler))
    // })
    // .bind("127.0.0.1:8088")?.bind("192.168.1.66:8088")?
    // .run()
    // .await
}
