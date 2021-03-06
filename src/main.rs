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

use structopt::StructOpt;

mod config;
mod keybind;
pub mod opts;
mod template;
mod ws;
mod soundboard_commands;

pub type ServerData = Arc<
    RwLock<(
        (
            Config,
            Arc<RwLock<SoundConfig>>,
            (
                mlws_lib::SoundSender,
                mlws_lib::SoundReceiver,
                mlws_lib::SoundLoop,
            ),
            config::ConfigClient,
        ),
        keybind::KeybindsClient, // Arc<Mutex<mlws_lib::keybind::KeyBindings<mlws_lib::sound::Message, F, (String, String)>>>
    )>,
>;

#[tokio::main]
async fn main() {
    let opts = opts::Opt::from_args();

    if let Some(command) = opts.subcommand {
        match command {
            opts::Subcommmand::Soundboard { subcommand } => soundboard_commands::handle(subcommand),
        }
    } else {
        println!("Setting up mlws");
        let (sound_sender, sound_receiver, mut soundloop) = mlws_lib::setup();
        println!("Running the sound loop");
        soundloop.run().expect("Error starting soundloop");
        println!("Loading config");
        let mut config = Config::load();
        println!("Loading sounds");
        let sounds = Arc::new(RwLock::new(SoundConfig::load(&mut config).await));
        config.save();
        let sounds_clone = sounds.clone();
        let mut keybinds = keybind::KeybindsServer::new(mlws_lib::keybind::KeyBindings::new(
            sound_sender.clone(),
            config.clone(),
            move |(repo, name)| {
                mlws_lib::sound::Message::PlaySound(
                    sounds_clone
                        .read()
                        .unwrap()
                        .get(&repo, &name)
                        .unwrap()
                        .clone(),
                    mlws_lib::sound::SoundDevices::Both,
                )
            },
        ));

        let mut config_server = config::ConfigServer::new();

        // {
        //     let conn = keybinds.connection();
        //     conn.add(
        //         ("Team Fortress 2".into(), "AAAAAAAAAA".into()),
        //         vec![mlws_lib::rdev::Key::KeyA],
        //     );
        // }

        let conn = keybinds.connection();
        let conf_conn = config_server.connection();

        println!("Starting server_client threads");
        std::thread::spawn(move || loop {
            keybinds.tick();
            config_server.tick();
        });

        let data = (
            conf_conn.load(),
            sounds,
            (sound_sender, sound_receiver, soundloop),
            conf_conn.clone(),
        );

        let data_idx = data.1.clone();
        let index = warp::path::end().map(move || {
            let mut ctx = Context::new();
            let mut repos = data_idx
                .read()
                .unwrap()
                .json_sounds()
                .iter()
                .map(|(a, b)| (a.clone(), b.clone()))
                .map(|(k, mut s)| {
                    s.sort();
                    (k, s)
                })
                .collect::<Vec<(String, Vec<String>)>>();
            repos.sort_by(|(a, _), (b, _)| a.cmp(b));

            ctx.insert("repos", &repos);
            warp::reply::html(template::render_context("index.html", &ctx))
        });

        let conf_sett = conf_conn.clone();
        let settings = warp::path!("settings").map(move || {
            let mut ctx = Context::new();
            let cfg = conf_sett.load();
            // ctx.insert("config", &cfg);
            ctx.insert("repos", &cfg.repos);
            warp::reply::html(template::render_context("settings.html", &ctx))
        });

        let css = warp::path!("css" / String).map(|p| {
            warp::reply::with_header(
                template::load(&format!("css/{}", url::decode(p))),
                "Content-Type",
                "text/css",
            )
        });

        let halfmoon_css = warp::path!("halfmoon" / "css" / String).map(|p| {
            warp::reply::with_header(
                template::load(&format!("halfmoon/css/{}", url::decode(p))),
                "Content-Type",
                "text/css",
            )
        });

        let halfmoon_js = warp::path!("halfmoon" / "js" / String).map(|p| {
            warp::reply::with_header(
                template::load(&format!("halfmoon/js/{}", url::decode(p))),
                "Content-Type",
                "text/javascript",
            )
        });

        let data_img = data.1.clone();
        let sound_img = warp::path!("sound" / String / String / "img").map(move |repo, name| {
            let repo = url::decode(repo);
            let name = url::decode(name);
            println!("REPO: {}; SOUND: {}", repo, name);
            match data_img.read().unwrap().get(&repo, &name) {
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

        //let data_repo = data.clone();
        let repo = warp::path!("repo" / usize).map(move |id: usize| {
            let cfg = mlws_lib::config::Config::load();
            if let Some((repo, down)) = cfg.repos.get(&id).map(|x| x.clone()) {
                let (s, r) = std::sync::mpsc::channel();
                let (repo_clone, down_clone) = (repo.clone(), down.clone());
                std::thread::spawn(move || {
                    let mut rt = tokio::runtime::Runtime::new().unwrap();
                    s.send(rt.block_on(async {
                        mlws_lib::downloader::status(&repo_clone, &down_clone).await
                    }))
                    .unwrap();
                });
                let status = r.recv().unwrap();
                let status_str = match status {
                    mlws_lib::downloader::Status::Latest(_) => format!("Latest"),
                    mlws_lib::downloader::Status::Updatable(_, latest) => format!(
                    "Updatable <code>{}</code> <button onclick=\"update_repo({})\">UPDATE</button>",
                    latest.trim(),
                    id
                ),
                };
                let mut ctx = Context::new();
                ctx.insert("repo", &(repo, down, status_str, id));
                warp::reply::html(template::render_context_no_escapes("repo.html", &ctx))
                    .into_response()
            } else {
                warp::http::StatusCode::NOT_FOUND.into_response()
            }
        });

        // let conf_key = data.0.clone();
        let sound_key = data.1.clone();
        let keybind_conn = conn.clone();
        let keybind = warp::path!("keybind" / usize).map(move |id: usize| {
            match &keybind_conn.keys()[id] {
                ((repo, name), keys) => {
                    let keys = keys
                        .iter()
                        .map(|x| format!("{}", x))
                        .collect::<Vec<String>>()
                        .join(" + ");
                    let mut ctx = Context::new();
                    let sounds: Vec<String> = sound_key
                        .read()
                        .unwrap()
                        .sounds
                        .get(repo)
                        .map(|x| x.keys().cloned().collect())
                        .unwrap_or_default();
                    ctx.insert("keybind", &((repo, name), keys, id));
                    let repos: Vec<String> = mlws_lib::config::Config::load()
                        .repos
                        .iter()
                        .map(|(_, name)| name)
                        .filter(|n| n.is_some())
                        .map(|n| n.clone().unwrap().name)
                        .collect();
                    ctx.insert("repos", &repos);
                    ctx.insert("sounds", &sounds);
                    warp::reply::html(template::render_context_no_escapes("keybind.html", &ctx))
                        .into_response()
                } // _ => warp::http::StatusCode::NOT_FOUND.into_response(),
            }
        });

        let ws = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let data_ws = Arc::new(RwLock::new((data.clone(), conn.clone())));
                let res = ws.on_upgrade(move |ws| ws::ws(ws, data_ws));
                res
            });

        let route = index
            .or(settings)
            .or(css)
            .or(halfmoon_css)
            .or(halfmoon_js)
            .or(sound_img)
            .or(ws)
            .or(repo)
            .or(keybind);

        for ip in opts.ip {
            let route = route.clone();
            let ip = format!("{}:{}", ip, opts.port);
            println!("IP: {}", ip);
            std::thread::spawn(move || {
                let mut rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(warp::serve(route).run(ip.parse::<std::net::SocketAddr>().unwrap()));
            });
        }

        loop {}
    }
}
