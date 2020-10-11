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
// use mlws_lib::rdev::Key;

#[derive(Debug, Serialize, Deserialize)]
enum WsIncomingMessage {
    Play(String, String),
    Stop(String, String),
    StopAll(),
    Status(),
    RepoNum(),
    RemoveRepo(usize),
    AddRepo(String, String),
    UpdateRepo(usize),
    KeybindNum(),
    Repos(),
    Sounds(String, String),
    AddKeybind(String, String),
    RemoveKeybind(usize),
    SetKeybind(usize, (String, String)),
    Detect(usize),
    StopDetect(),
    HasDetected(),
    SaveKeybinds(),
}

#[derive(Debug, Serialize, Deserialize)]
enum WsOutgoingMessage {
    Status(Vec<((String, String), Duration, Option<Duration>)>),
    RepoNum(Vec<usize>),
    RepoReload(usize),
    Downloading(usize, u64, bool),
    Installing(usize),
    Done(usize),
    KeybindNum(Vec<usize>),
    Repos(Vec<String>),
    Sounds(String, Vec<String>),
    HasDetected(String),
}

pub async fn ws(ws: warp::ws::WebSocket, mut data: ServerData) {
    let (mut tx, mut rx) = ws.split();
    // let (send, recv) = mpsc::channel();
    while let Some(msg) = rx.next().await {
        if let Ok(msg) = msg.map_err(|x| println!("{}", x)).and_then(|x| {
            // println!("{:?}", x);
            x.to_str().and_then(|x| {
                serde_json::from_str::<WsIncomingMessage>(x).map_err(|x| println!("{}", x))
            })
        }) {
            handle(&mut data, msg, &mut tx).await
        }
    }
}

async fn send(tx: &mut SplitSink<warp::ws::WebSocket, warp::ws::Message>, msg: WsOutgoingMessage) {
    // println!("{:?}", msg);
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
    let ((_, sound_cfg, (sender, receiver, _), conf_client), keybinds) =
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
            let cfg = conf_client.load();

            send(
                tx,
                WsOutgoingMessage::RepoNum(cfg.repos.ids().map(|x| *x).collect()),
            )
            .await;
        }
        WsIncomingMessage::KeybindNum() => {
            // let (s, r) = mpsc::channel();
            // let (repo, data) = &config.repos[i];

            send(tx, WsOutgoingMessage::KeybindNum(keybinds.ids())).await;
        }
        WsIncomingMessage::UpdateRepo(i) => {
            println!("Updating repo {}", i);
            let (s2, mut r2) = mpsc::unbounded_channel();
            let (s, mut r) = mpsc::unbounded_channel();

            let mut config = conf_client.load();

            println!("UPDATE");
            // std::thread::spawn(move || {
            //     actix_rt::Runtime::new().unwrap().block_on(download(s, s2, repo, data, i));
            // });
            let repos_clone = conf_client.load().repos[i].clone();
            tokio::spawn(async move {
                let (repo, mut down) = repos_clone;
                mlws_lib::downloader::download(
                    &repo,
                    &mut down,
                    move |p| {
                        // println!("{:?}", p);
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
                                mlws_lib::downloader::Progress::Done() => {
                                    WsOutgoingMessage::Done(i)
                                }
                            })
                            .expect("Error sending data");
                    },
                    true,
                )
                .await;
                s2.send((repo, down)).unwrap();
            });

            loop {
                if let Ok(v) = r.try_recv() {
                    // println!("RECV: {:?}", v);
                    if let WsOutgoingMessage::Done(_) = &v {
                        send(tx, v).await;
                        break;
                    }
                    send(tx, v).await;
                }
                tokio::task::yield_now().await;
            }

            config.repos[i] = r2.recv().await.expect("Error receiving data");

            conf_client.save(config);

            // let cfg = config.clone();

            // let status = status(cfg, i).await;

            send(tx, WsOutgoingMessage::RepoReload(i)).await;
            // handle(data, WsIncomingMessage::RepoStatus(i), tx).await;
            //self.handle(Ok(ws::Message::Text(serde_json::to_string(&WsIncomingMessage::RepoStatus(i)).unwrap())), ctx);
        }
        WsIncomingMessage::RemoveRepo(id) => {
            println!("Removing {}", id);
            let mut conf: mlws_lib::config::Config = conf_client.load();
            conf.repos.remove(id);

            let ids = conf.repos.ids().map(|x| *x).collect();
            conf_client.save(conf);
            println!("Repo num: {:?}", ids);
            send(tx, WsOutgoingMessage::RepoNum(ids)).await;
        }
        WsIncomingMessage::AddRepo(zip_url, version_url) => {
            let mut conf: mlws_lib::config::Config = conf_client.load();
            conf.repos
                .add((mlws_lib::config::SoundRepo::new(zip_url, version_url), None));

            let ids = conf.repos.ids().map(|x| *x).collect();
            conf_client.save(conf);

            send(tx, WsOutgoingMessage::RepoNum(ids)).await;
        }
        WsIncomingMessage::Repos() => {
            let conf: mlws_lib::config::Config = conf_client.load();
            let mut cfg: Vec<String> = conf
                .repos
                .iter()
                .map(|(_, name)| name)
                .filter(|n| n.is_some())
                .map(|n| n.clone().unwrap().name)
                .collect();
            cfg.sort();
            send(tx, WsOutgoingMessage::Repos(cfg)).await;
        }
        WsIncomingMessage::Sounds(repo, i) => {
            let cfg: Vec<String> = sound_cfg
                .read()
                .unwrap()
                .sounds
                .get(&repo)
                .map(|x| {
                    let mut x: Vec<String> = x.keys().cloned().collect();
                    x.sort();
                    x
                })
                .unwrap_or_default();
            send(tx, WsOutgoingMessage::Sounds(i, cfg)).await;
        }
        WsIncomingMessage::AddKeybind(repo, sound) => {
            keybinds.add((repo, sound));
            send(tx, WsOutgoingMessage::KeybindNum(keybinds.ids())).await;
        }
        WsIncomingMessage::RemoveKeybind(i) => {
            keybinds.remove(i);
            send(tx, WsOutgoingMessage::KeybindNum(keybinds.ids())).await;
        }
        WsIncomingMessage::Detect(i) => {
            keybinds.detect(i);
        }
        WsIncomingMessage::StopDetect() => {
            keybinds.stop_detect();
            send(tx, WsOutgoingMessage::KeybindNum(keybinds.ids())).await;
        }
        WsIncomingMessage::HasDetected() => {
            if let Some(detected) = keybinds.has_detected() {
                send(
                    tx,
                    WsOutgoingMessage::HasDetected(
                        detected
                            .iter()
                            .map(|x| format!("{}", x))
                            .collect::<Vec<String>>()
                            .join(" + "),
                    ),
                )
                .await;
            }
        }
        WsIncomingMessage::SetKeybind(id, sound) => {
            keybinds.set(id, sound);
        }
        WsIncomingMessage::SaveKeybinds() => {
            println!("Saving keybinds");
            let mut conf: mlws_lib::config::Config = conf_client.load();
            conf.hotkeys = keybinds.save_config();
            conf_client.save(conf);
        }
    }
}
