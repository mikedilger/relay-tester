use crate::error::Error;
use crate::PREFIXES;
use super::{Command, url_to_host_and_uri};
use base64::Engine;
use colorful::{Color, Colorful};
use futures_util::stream::FusedStream;
use futures_util::{SinkExt, StreamExt};
use nostr_types::{ClientMessage, RelayMessage};
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tungstenite::Message;

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Debug)]
pub struct ProbeInner {
    pub input: Receiver<Command>,
    pub output: Sender<String>,
}

impl ProbeInner {
    pub async fn connect_and_listen(&mut self, relay_url: &str) -> Result<(), Error> {
        let (host, uri) = url_to_host_and_uri(relay_url);

        let key: [u8; 16] = rand::random();
        let request = http::request::Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                base64::engine::general_purpose::STANDARD.encode(key),
            )
            .uri(uri)
            .body(())?;

        let (mut websocket, _response) = tokio::time::timeout(
            Duration::new(5, 0),
            tokio_tungstenite::connect_async(request),
        )
        .await??;

        let mut ping_timer = tokio::time::interval(Duration::new(15, 0));
        ping_timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        ping_timer.tick().await; // use up the first immediate tick.

        loop {
            tokio::select! {
                _ = ping_timer.tick() => {
                    let msg = Message::Ping(vec![0x1]);
                    self.send(&mut websocket, msg).await;
                },
                local_message = self.input.recv() => {
                    match local_message {
                        Some(Command::PostEvent(event)) => {
                            let client_message = ClientMessage::Event(Box::new(event));
                            let wire = serde_json::to_string(&client_message)?;
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::PostRawEvent(event)) => {
                            let wire = format!("[\"EVENT\",{}]", event);
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::Auth(event)) => {
                            let client_message = ClientMessage::Auth(Box::new(event));
                            let wire = serde_json::to_string(&client_message)?;
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::FetchEvents(subid, filters)) => {
                            let client_message = ClientMessage::Req(subid, filters);
                            let wire = serde_json::to_string(&client_message)?;
                            let msg = Message::Text(wire);
                            self.send(&mut websocket, msg).await;
                        },
                        Some(Command::Exit) => {
                            self.send(&mut websocket, Message::Close(None)).await;
                        },
                        None => { }
                    }
                },
                message = websocket.next() => {
                    let message = match message {
                        Some(m) => m,
                        None => {
                            if websocket.is_terminated() {
                                eprintln!("{}", "Connection terminated".color(Color::Orange1));
                            }
                            break;
                        }
                    }?;

                    // Display it
                    Self::display(message.clone())?;

                    // Take action
                    match message {
                        Message::Text(s) => self.output.send(s).await.unwrap(),
                        Message::Binary(_) => { },
                        Message::Ping(_) => { },
                        Message::Pong(_) => { },
                        Message::Close(_) => break,
                        Message::Frame(_) => unreachable!(),
                    }
                },
            }
        }

        Ok(())
    }

    fn display(message: Message) -> Result<(), Error> {
        match message {
            Message::Text(s) => {
                let relay_message: RelayMessage = serde_json::from_str(&s)?;
                match relay_message {
                    RelayMessage::Auth(challenge) => {
                        eprintln!("{}: AUTH({})", PREFIXES.from_relay, challenge);
                    }
                    RelayMessage::Event(sub, e) => {
                        let event_json = serde_json::to_string(&e)?;
                        eprintln!(
                            "{}: EVENT({}, {})",
                            PREFIXES.from_relay,
                            sub.as_str(),
                            event_json
                        );
                    }
                    RelayMessage::Closed(sub, msg) => {
                        eprintln!("{}: CLOSED({}, {})", PREFIXES.from_relay, sub.as_str(), msg);
                    }
                    RelayMessage::Notice(s) => {
                        eprintln!("{}: NOTICE({})", PREFIXES.from_relay, s);
                    }
                    RelayMessage::Eose(sub) => {
                        eprintln!("{}: EOSE({})", PREFIXES.from_relay, sub.as_str());
                    }
                    RelayMessage::Ok(id, ok, reason) => {
                        eprintln!(
                            "{}: OK({}, {}, {})",
                            PREFIXES.from_relay,
                            id.as_hex_string(),
                            ok,
                            reason
                        );
                    }
                }
            }
            Message::Binary(_) => {
                eprintln!("{}: Binary message received!!!", PREFIXES.from_relay);
            }
            Message::Ping(_) => {
                eprintln!("{}: Ping", PREFIXES.from_relay);
            }
            Message::Pong(_) => {
                eprintln!("{}: Pong", PREFIXES.from_relay);
            }
            Message::Close(_) => {
                eprintln!("{}", "Remote closed nicely.".color(Color::Green));
            }
            Message::Frame(_) => {
                unreachable!()
            }
        }

        Ok(())
    }

    async fn send(&mut self, websocket: &mut Ws, message: Message) {
        match message {
            Message::Text(ref s) => eprintln!("{}: Text({})", PREFIXES.sending, s),
            Message::Binary(_) => eprintln!("{}: Binary(_)", PREFIXES.sending),
            Message::Ping(_) => eprintln!("{}: Ping(_)", PREFIXES.sending),
            Message::Pong(_) => eprintln!("{}: Pong(_)", PREFIXES.sending),
            Message::Close(_) => eprintln!("{}: Close(_)", PREFIXES.sending),
            Message::Frame(_) => eprintln!("{}: Frame(_)", PREFIXES.sending),
        }
        websocket.send(message).await.unwrap()
    }
}
