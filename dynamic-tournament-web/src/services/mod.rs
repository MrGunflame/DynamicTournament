pub mod client;
pub mod errorlog;

use std::collections::HashSet;

use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::Frame;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{Agent, AgentLink, Context, Dispatched, Dispatcher, HandlerId};

use dynamic_tournament_api::websocket::{EventHandler, WebSocket};
use dynamic_tournament_api::Client;

use gloo_utils::errors::JsError;

#[derive(Clone, Debug)]
pub struct WebSocketService {
    ws: WebSocket<Frame>,
}

impl WebSocketService {
    pub fn new(
        client: &Client,
        tournament_id: TournamentId,
        bracket_id: BracketId,
    ) -> Result<Self, JsError> {
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
                ws.send(&Frame::Authorize(auth)).await.unwrap();
            });
        }

        Ok(Self { ws })
    }

    pub async fn send(&mut self, msg: Frame) {
        log::debug!("Sending frame: {:?}", msg);

        let _ = self.ws.send(&msg).await;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    EventBusMsg(Frame),
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

struct Handler(Dispatcher<EventBus>);

impl EventHandler<Frame> for Handler {
    fn dispatch(&mut self, msg: Frame) {
        log::debug!("Received frame: {:?}", msg);

        self.0.send(Request::EventBusMsg(msg));
    }
}
