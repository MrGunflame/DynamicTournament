pub mod client;

use std::collections::HashSet;

use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use reqwasm::websocket::futures::WebSocket;
use reqwasm::websocket::Message;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{Agent, AgentLink, Context, Dispatched, HandlerId};

use dynamic_tournament_api::websocket;
use dynamic_tournament_api::Client;

const BUFFER_SIZE: usize = 32;

#[derive(Clone, Debug)]
pub struct WebSocketService {
    tx: mpsc::Sender<websocket::Message>,
}

impl WebSocketService {
    pub fn new(client: Client, id: u64) -> Self {
        // Replace http with ws and https with wss.
        let uri = format!(
            "{}/v2/tournament/{}/bracket",
            client.base_url().replacen("http", "ws", 1),
            id
        );

        let auth = client.authorization().auth_token().map(|s| s.to_owned());

        let ws = WebSocket::open(&uri).unwrap();

        let (mut writer, mut reader) = ws.split();

        log::debug!("Connecting to {}", uri);

        let (tx, mut rx) = mpsc::channel::<websocket::Message>(BUFFER_SIZE);
        let mut event_bus = EventBus::dispatcher();

        spawn_local(async move {
            if let Some(auth) = auth {
                let msg = websocket::Message::Authorize(auth).into_bytes();
                writer.send(Message::Bytes(msg)).await.unwrap();
            }

            while let Some(msg) = rx.next().await {
                log::debug!("Sending message to ws peer: {:?}", msg);
                let enc = msg.into_bytes();
                log::debug!("Serialized buffer: {:?} ({} bytes)", enc, enc.len());

                writer.send(Message::Bytes(enc)).await.unwrap();
            }

            // Dropping all sender will close the connection.
            log::debug!("Dropping ws writer");
            let _ = writer.close().await;
        });

        spawn_local(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Bytes(buf)) => {
                        log::debug!("Received message from ws peer: {:?}", buf);
                        let msg = websocket::Message::from_bytes(&buf).unwrap();
                        event_bus.send(Request::EventBusMsg(msg));
                    }
                    Ok(Message::Text(_)) => panic!("cannot read text"),
                    Err(err) => {
                        log::error!("{:?}", err);
                    }
                }
            }

            log::debug!("Closing ws connection");
        });

        log::debug!("Connected to {}", uri);

        Self { tx }
    }

    pub async fn send(&mut self, msg: websocket::Message) {
        self.tx.send(msg).await;
    }

    /// Returns `true` is the websocket is closed.
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    EventBusMsg(websocket::Message),
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = websocket::Message;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            Request::EventBusMsg(msg) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, msg.clone());
                }
            }
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}
