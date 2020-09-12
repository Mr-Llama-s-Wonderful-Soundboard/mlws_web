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
    stream::{SplitSink, SplitStream, StreamExt},
};

use std::time::Duration;

use mlws_lib;
use warp::Filter;

use std::marker::Sync;

#[derive(Debug, Serialize, Deserialize)]
enum WsIncomingMessage {
    Play(String, String),
    Stop(String, String),
    StopAll(),
    Status(),
    RepoStatus(usize),
    UpdateRepo(usize),
}

#[derive(Debug, Serialize, Deserialize)]
enum WsOutgoingMessage {
    Status(Vec<((String, String), Duration, Option<Duration>)>),
    RepoStatus(usize, mlws_lib::downloader::Status),
    Downloading(usize, u64, bool),
    Installing(usize),
    Done(usize),
}

// impl Handler<WsIncomingMessage> for Ws {
//     type Result = ();
//     fn handle(&mut self, msg: WsIncomingMessage, ctx: &mut Self::Context) -> Self::Result {
//         let (config, sound_cfg, (sender, receiver, _)) = &mut self.data;
//         match msg {
//             WsIncomingMessage::Play(repo, name) => {
//                 if let Some(sound) = sound_cfg.get(&repo, &name) {
//                     sender
//                         .send(mlws_lib::sound::Message::PlaySound(
//                             sound.clone(),
//                             mlws_lib::sound::SoundDevices::Both,
//                         ))
//                         .expect("Error sending message");
//                 }
//             }
//             WsIncomingMessage::Stop(repo, name) => {
//                 if let Some(sound) = sound_cfg.get(&repo, &name) {
//                     sender
//                         .send(mlws_lib::sound::Message::StopSound(sound.clone()))
//                         .expect("Error sending message");
//                 }
//             }
//             WsIncomingMessage::StopAll() => sender
//                 .send(mlws_lib::sound::Message::StopAll)
//                 .expect("Error sending messagee"),
//             WsIncomingMessage::Status() => {
//                 sender
//                     .send(mlws_lib::sound::Message::PlayStatus(vec![], 0.0))
//                     .expect("Error sending message");
//                 if let mlws_lib::sound::Message::PlayStatus(status, _) =
//                     receiver.recv().expect("Error receiving")
//                 {
//                     let new_status = status
//                         .iter()
//                         .cloned()
//                         .map(|(_, s, d, t)| ((s.repo, s.name), d, t))
//                         .collect();

//                     ctx.text(
//                         serde_json::to_string(&WsOutgoingMessage::Status(new_status))
//                             .expect("Error converting status"),
//                     );
//                 }
//             }
//             WsIncomingMessage::RepoStatus(i) => {
//                 let (s, r) = mpsc::channel();
//                 // let (repo, data) = &config.repos[i];
//                 let cfg = config.clone();
//                 std::thread::spawn(move || {
//                     s.send(actix_rt::Runtime::new().unwrap().block_on(status(cfg, i)))
//                 });
//                 let status: mlws_lib::downloader::Status = r.recv().unwrap();
//                 println!("Status: {:?}", status);
//                 ctx.text(
//                     serde_json::to_string(&WsOutgoingMessage::RepoStatus(i, status))
//                         .expect("Error converting status"),
//                 );
//             }
//             WsIncomingMessage::UpdateRepo(i) => {
//                 let (s, r) = mpsc::channel();
//                 let (s2, r2) = mpsc::channel();
//                 let (repo, data) = config.repos[i].clone();
//                 println!("UPDATE");
//                 // std::thread::spawn(move || {
//                 //     actix_rt::Runtime::new().unwrap().block_on(download(s, s2, repo, data, i));
//                 // });
//                 mlws_lib::downloader::download_threaded(
//                     repo.clone(),
//                     data.clone(),
//                     move |p| {
//                         println!("{:?}", p);
//                         s2.send(match p {
//                             mlws_lib::downloader::Progress::Downloading(a, l) => {
//                                 if let Some(len) = l {
//                                     WsOutgoingMessage::Downloading(
//                                         i,
//                                         (a as f64 * 100. / len as f64) as u64,
//                                         true,
//                                     )
//                                 } else {
//                                     WsOutgoingMessage::Downloading(i, a, false)
//                                 }
//                             }
//                             mlws_lib::downloader::Progress::Installing() => {
//                                 WsOutgoingMessage::Installing(i)
//                             }
//                             mlws_lib::downloader::Progress::Done() => WsOutgoingMessage::Done(i),
//                         })
//                         .expect("Error sending message")
//                     },
//                     s,
//                 );

//                 loop {
//                     if let Ok(v) = r2.try_recv() {
//                         println!("RECV: {:?}", v);
//                         ctx.text(serde_json::to_string(&v).unwrap());

//                         if let WsOutgoingMessage::Done(_) = v {
//                             break;
//                         }
//                     }
//                 }
//                 config.repos[i].1 = r.recv().unwrap();
//                 actix::Handler::handle(self, WsIncomingMessage::RepoStatus(i), ctx);
//                 //self.handle(Ok(ws::Message::Text(serde_json::to_string(&WsIncomingMessage::RepoStatus(i)).unwrap())), ctx);
//             }
//         }
//     }
// }

// /// Handler for ws::Message message
// impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
//     fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
//         let (config, sound_cfg, (sender, receiver, _)) = &mut self.data;
//         match msg {
//             Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
//             Ok(ws::Message::Text(text)) => {
//                 println!("WS: {}", text);
//                 actix::Handler::handle(self, serde_json::from_str::<WsIncomingMessage>(&text).expect("Unexpected message"), ctx);
//             }
//             Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
//             _ => (),
//         }
//     }
// }

async fn status(config: mlws_lib::config::Config, i: usize) -> mlws_lib::downloader::Status {
    let (repo, data) = &config.repos[i];
    println!("STATUS");
    mlws_lib::downloader::status(repo, data).await
}

// async fn download(
//     s: mpsc::Sender<Option<mlws_lib::config::DownloadedSoundRepo>>,
//     s2: mpsc::Sender<WsOutgoingMessage>,
//     repo: mlws_lib::config::SoundRepo,
//     mut data: Option<mlws_lib::config::DownloadedSoundRepo>,
//     i: usize,
// ) {
//     mlws_lib::downloader::download(&repo, &mut data, |p| {
//         println!("Downloader: {:?}", p);
//         s2.send(match p {
//             mlws_lib::downloader::Progress::Downloading(a, l) => {
//                 if let Some(len) = l {
//                     WsOutgoingMessage::Downloading(i, (a as f64 * 100. / len as f64) as u64, true)
//                 } else {
//                     WsOutgoingMessage::Downloading(i, a, false)
//                 }
//             }
//             mlws_lib::downloader::Progress::Installing() => WsOutgoingMessage::Installing(i),
//             mlws_lib::downloader::Progress::Done() => WsOutgoingMessage::Done(i),
//         })
//         .expect("Error sending message")
//     })
//     .await;
//     s.send(data).unwrap();
// }
// use futures::{stream::{StreamExt, SplitSink, SplitStream}, sink::SinkExt};
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
    let (config, sound_cfg, (sender, receiver, _)) =
        std::sync::Arc::get_mut(data).unwrap().get_mut().unwrap();
    match msg {
        WsIncomingMessage::Play(repo, name) => {
            if let Some(sound) = sound_cfg.get(&repo, &name) {
                sender
                    .send(mlws_lib::sound::Message::PlaySound(
                        sound.clone(),
                        mlws_lib::sound::SoundDevices::Both,
                    ))
                    .expect("Error sending message");
            }
        }
        WsIncomingMessage::Stop(repo, name) => {
            if let Some(sound) = sound_cfg.get(&repo, &name) {
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
        WsIncomingMessage::RepoStatus(i) => {
            // let (s, r) = mpsc::channel();
            // let (repo, data) = &config.repos[i];
            let cfg = config.clone();

            let status = status(cfg, i).await;
            println!("Status: {:?}", status);
            send(tx, WsOutgoingMessage::RepoStatus(i, status)).await;
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

                }).await;
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

            let cfg = config.clone();

            let status = status(cfg, i).await;

            send(tx, WsOutgoingMessage::RepoStatus(i, status)).await;
            // handle(data, WsIncomingMessage::RepoStatus(i), tx).await;
            //self.handle(Ok(ws::Message::Text(serde_json::to_string(&WsIncomingMessage::RepoStatus(i)).unwrap())), ctx);
        }
    }
}
