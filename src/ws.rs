use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};
use serde_json;

use crate::ServerData;

use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
enum WsIncomingMessage {
    Play(String, String),
    Stop(String, String),
    StopAll(),
    Status(),
}

/// Define http actor
struct Ws {
    data: ServerData,
}

impl Actor for Ws {
    type Context = ws::WebsocketContext<Self>;
}

#[derive(Debug, Serialize, Deserialize)]
enum WsOutgoingMessage {
    Status(Vec<((String, String), Duration, Option<Duration>)>),
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Ws {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let (_, sound_cfg, (sender, receiver, _)) = &self.data;
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<WsIncomingMessage>(&text).expect("Unexpected message")
                {
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

                            ctx.text(
                                serde_json::to_string(&WsOutgoingMessage::Status(new_status))
                                    .expect("Error converting status"),
                            );
                        }
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

pub async fn ws(
    data: web::Data<ServerData>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        Ws {
            data: data.as_ref().clone(),
        },
        &req,
        stream,
    );println!("{}", serde_json::to_string(&WsIncomingMessage::Status())
	.expect("Error converting status"));
    println!("{:?}", resp);
    resp
}
