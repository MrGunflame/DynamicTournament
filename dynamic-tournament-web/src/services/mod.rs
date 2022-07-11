pub mod client;
pub mod errorlog;

use std::collections::HashSet;

use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::Frame;
use dynamic_tournament_api::Error;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{Agent, AgentLink, Context, Dispatched, Dispatcher, HandlerId};

use dynamic_tournament_api::websocket::{EventHandler, WebSocket, WebSocketMessage};
use dynamic_tournament_api::Client;

pub use errorlog::MessageLog;

#[derive(Clone, Debug)]
pub struct WebSocketService {
    ws: WebSocket,
}

impl WebSocketService {
    pub fn new(
        client: &Client,
        tournament_id: TournamentId,
        bracket_id: BracketId,
    ) -> Result<Self, Error> {
        let builder = client
            .v3()
            .tournaments()
            .brackets(tournament_id)
            .matches(bracket_id)
            .handler(Box::new(Handler(EventBus::dispatcher())));

        let auth = client.authorization().auth_token().map(|s| s.to_owned());

        let ws = builder.build()?;

        if let Some(auth) = auth {
            let mut ws = ws.clone();
            spawn_local(async move {
                let msg = Frame::Authorize(auth).to_bytes().unwrap();

                ws.send(msg).await;
            });
        }

        Ok(Self { ws })
    }

    pub async fn send(&mut self, msg: Frame) {
        log::debug!("Sending frame: {:?}", msg);

        let msg = msg.to_bytes().unwrap();
        self.ws.send(msg).await;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Message(Frame),
    Close,
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Request;
    type Output = Frame;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        match msg {
            Request::Message(msg) => {
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, msg.clone());
                }
            }
            Request::Close => {}
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

struct Handler(Dispatcher<EventBus>);

impl EventHandler for Handler {
    fn dispatch(&mut self, msg: WebSocketMessage) {
        log::debug!("Received frame: {:?}", msg);

        match msg {
            WebSocketMessage::Bytes(buf) => self
                .0
                .send(Request::Message(Frame::from_bytes(&buf).unwrap())),
            WebSocketMessage::Text(_) => (),
            WebSocketMessage::Close => self.0.send(Request::Close),
        }
    }
}
