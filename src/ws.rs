// use actix::{Actor, Handler, Message, StreamHandler};
// use actix_rt;
// use actix_web::{web, App, Error, HttpRequest, HttpResponse};
// use actix_web_actors::ws;

use serde::{Deserialize, Serialize};
use serde_json;

use tokio::sync::mpsc;

use crate::ServerData;

use futures::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
};

use std::time::Duration;

use mlws_lib;

#[derive(Debug, Serialize, Deserialize)]
enum WsIncomingMessage {
    Play(String, String),
    Stop(String, String),
    StopAll(),
    Status(),
    RepoNum(),
    UpdateRepo(usize),
    KeybindNum(),
    Repos(),
    Sounds(String, String),
    AddKeyBind(),
}

#[derive(Debug, Serialize, Deserialize)]
enum WsOutgoingMessage {
    Status(Vec<((String, String), Duration, Option<Duration>)>),
    RepoNum(usize),
    RepoReload(usize),
    Downloading(usize, u64, bool),
    Installing(usize),
    Done(usize),
    KeybindNum(usize),
    Repos(Vec<String>),
    Sounds(String, Vec<String>),
}

// async fn status(config: mlws_lib::config::Config, i: usize) -> mlws_lib::downloader::Status {
//     let (repo, data) = &config.repos[i];
//     println!("STATUS");
//     mlws_lib::downloader::status(repo, data).await
// }

pub async fn ws(ws: warp::ws::WebSocket, mut data: ServerData) {
    let (mut tx, mut rx) = ws.split();
    // let (send, recv) = mpsc::channel();
    while let Some(msg) = rx.next().await {
        if let Ok(msg) = msg.map_err(|x| println!("{}", x)).and_then(|x| {
            x.to_str().and_then(|x| {
                serde_json::from_str::<WsIncomingMessage>(x).map_err(|x| println!("{}", x))
            })
        }) {
            handle(&mut data, msg, &mut tx).await
        }
    }
}

async fn send(tx: &mut SplitSink<warp::ws::WebSocket, warp::ws::Message>, msg: WsOutgoingMessage) {
    println!("{:?}", msg);
    tx.send(warp::ws::Message::text(
        serde_json::to_string(&msg).expect("Error receiving data"),
    ))
    .await
    .unwrap()
}

async fn handle(
    data: &mut ServerData,
    msg: WsIncomingMessage,
    tx: &mut SplitSink<warp::ws::WebSocket, warp::ws::Message>,
) {
    let ((config, sound_cfg, (sender, receiver, _)), keybinds) =
        std::sync::Arc::get_mut(data).unwrap().get_mut().unwrap();
    match msg {
        WsIncomingMessage::Play(repo, name) => {
            if let Some(sound) = sound_cfg.read().unwrap().get(&repo, &name) {
                sender
                    .send(mlws_lib::sound::Message::PlaySound(
                        sound.clone(),
                        mlws_lib::sound::SoundDevices::Both,
                    ))
                    .expect("Error sending message");
            }
        }
        WsIncomingMessage::Stop(repo, name) => {
            if let Some(sound) = sound_cfg.read().unwrap().get(&repo, &name) {
                sender
                    .send(mlws_lib::sound::Message::StopSound(sound.clone()))
                    .expect("Error sending message");
            }
        }
        WsIncomingMessage::StopAll() => sender
            .send(mlws_lib::sound::Message::StopAll)
            .expect("Error sending messagee"),
        WsIncomingMessage::Status() => {
            sender
                .send(mlws_lib::sound::Message::PlayStatus(vec![], 0.0))
                .expect("Error sending message");
            if let mlws_lib::sound::Message::PlayStatus(status, _) =
                receiver.recv().expect("Error receiving")
            {
                let new_status = status
                    .iter()
                    .cloned()
                    .map(|(_, s, d, t)| ((s.repo, s.name), d, t))
                    .collect();

                send(tx, WsOutgoingMessage::Status(new_status)).await;
            }
        }
        WsIncomingMessage::RepoNum() => {
            // let (s, r) = mpsc::channel();
            // let (repo, data) = &config.repos[i];
            let cfg = config.clone();

            send(tx, WsOutgoingMessage::RepoNum(cfg.repos.len())).await;
        }
        WsIncomingMessage::KeybindNum() => {
            // let (s, r) = mpsc::channel();
            // let (repo, data) = &config.repos[i];

            send(tx, WsOutgoingMessage::KeybindNum(keybinds.keys().len())).await;
        }
        WsIncomingMessage::UpdateRepo(i) => {
            let (s2, mut r2) = mpsc::unbounded_channel();
            let (s, mut r) = mpsc::unbounded_channel();

            println!("UPDATE");
            // std::thread::spawn(move || {
            //     actix_rt::Runtime::new().unwrap().block_on(download(s, s2, repo, data, i));
            // });
            let repos_clone = config.repos[i].clone();
            let j = tokio::spawn(async move {
                let (repo, mut down) = repos_clone;
                mlws_lib::downloader::download(&repo, &mut down, move |p| {
                    println!("{:?}", p);
                    s.clone()
                        .send(match p {
                            mlws_lib::downloader::Progress::Downloading(a, l) => {
                                if let Some(len) = l {
                                    WsOutgoingMessage::Downloading(
                                        i,
                                        (a as f64 * 100. / len as f64) as u64,
                                        true,
                                    )
                                } else {
                                    WsOutgoingMessage::Downloading(i, a, false)
                                }
                            }
                            mlws_lib::downloader::Progress::Installing() => {
                                WsOutgoingMessage::Installing(i)
                            }
                            mlws_lib::downloader::Progress::Done() => WsOutgoingMessage::Done(i),
                        })
                        .expect("Error sending data");
                })
                .await;
                s2.send((repo, down)).unwrap();
            });

            loop {
                if let Ok(v) = r.try_recv() {
                    println!("RECV: {:?}", v);
                    if let WsOutgoingMessage::Done(_) = &v {
                        send(tx, v).await;
                        break;
                    }
                    send(tx, v).await;
                }
                tokio::task::yield_now().await;
            }

            config.repos[i] = r2.recv().await.expect("Error receiving data");

            config.save();

            // let cfg = config.clone();

            // let status = status(cfg, i).await;

            send(tx, WsOutgoingMessage::RepoReload(i)).await;
            // handle(data, WsIncomingMessage::RepoStatus(i), tx).await;
            //self.handle(Ok(ws::Message::Text(serde_json::to_string(&WsIncomingMessage::RepoStatus(i)).unwrap())), ctx);
        }
        WsIncomingMessage::Repos() => {
            let cfg: Vec<String> = config
                .repos
                .iter()
                .map(|(_, name)| name)
                .filter(|n| n.is_some())
                .map(|n| n.clone().unwrap().name)
                .collect();
            send(tx, WsOutgoingMessage::Repos(cfg)).await;
        }
        WsIncomingMessage::Sounds(repo, i) => {
            let cfg: Vec<String> = sound_cfg
                .read()
                .unwrap()
                .sounds
                .get(&repo)
                .map(|x| x.keys().cloned().collect())
                .unwrap_or_default();
            send(tx, WsOutgoingMessage::Sounds(i, cfg)).await;
        }
        WsIncomingMessage::AddKeyBind() => {
            keybinds.add((String::new(), String::new()), vec![]);
            send(tx, WsOutgoingMessage::KeybindNum(keybinds.keys().len())).await;
        }
    }
}
