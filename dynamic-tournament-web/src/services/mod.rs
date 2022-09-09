pub mod errorlog;

use std::collections::HashSet;
use std::io::Cursor;
use std::rc::Rc;
use std::time::Duration;

use asyncsync::local::Notify;
use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::{Decode, Request, Response};
use futures::channel::{mpsc, oneshot};
use futures::{SinkExt, StreamExt};
use gloo_timers::future::sleep;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use yew_agent::{Agent, AgentLink, Context, Dispatched, Dispatcher, HandlerId};

use dynamic_tournament_api::websocket::{EventHandler, WebSocketError, WebSocketMessage};
use dynamic_tournament_api::Client;

pub use errorlog::MessageLog;

#[derive(Clone, Debug)]
pub struct WebSocketService {
    #[allow(clippy::type_complexity)]
    tx: mpsc::Sender<(Vec<u8>, oneshot::Sender<Result<(), WebSocketError>>)>,
}

impl WebSocketService {
    pub fn new(client: &Client, tournament_id: TournamentId, bracket_id: BracketId) -> Self {
        let client = client.clone();

        let (tx, mut rx) =
            mpsc::channel::<(Vec<u8>, oneshot::Sender<Result<(), WebSocketError>>)>(32);

        spawn_local(async move {
            let close_waker = Rc::new(Notify::new());

            loop {
                let builder = client
                    .v3()
                    .tournaments()
                    .brackets(tournament_id)
                    .matches(bracket_id)
                    .handler(Box::new(Handler {
                        tournament_id,
                        bracket_id,
                        dispatcher: EventBus::dispatcher(),
                        close_waker: close_waker.clone(),
                    }));

                match builder.build() {
                    Ok(mut ws) => {
                        let auth = client.authorization().auth_token().map(|s| s.to_owned());

                        if let Some(auth) = auth {
                            let msg = Request::Authorize(auth.into_token()).to_bytes();

                            let _ = ws.send(msg).await;
                        }

                        EventBus::dispatcher().send(Message::Connect(tournament_id, bracket_id));

                        loop {
                            futures::select! {
                                _ = close_waker.notified() => {
                                    break;
                                }
                                msg = rx.next() => {
                                    match msg {
                                        Some((msg, tx)) => {
                                            let res = ws.send(WebSocketMessage::Bytes(msg)).await;
                                            let _ = tx.send(res);
                                        }
                                        None => return,
                                    }

                                }
                            }
                        }
                    }
                    Err(err) => {
                        log::error!("Failed to connect to websocket: {}", err);
                    }
                }

                log::debug!("Retrying connect in 15s");
                sleep(Duration::new(15, 0)).await;
            }
        });

        Self { tx }
    }

    pub async fn send(&mut self, req: Request) -> Result<(), WebSocketError> {
        log::debug!("Sending frame: {:?}", req);

        let msg = req.to_bytes();

        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send((msg, tx)).await;
        rx.await.unwrap()
    }
}

impl PartialEq for WebSocketService {
    fn eq(&self, other: &Self) -> bool {
        self.tx.same_receiver(&other.tx)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Response(Response),
    Connect(TournamentId, BracketId),
    Close(TournamentId, BracketId),
}

pub struct EventBus {
    link: AgentLink<EventBus>,
    subscribers: HashSet<HandlerId>,
}

impl Agent for EventBus {
    type Reach = Context<Self>;
    type Message = ();
    type Input = Message;
    type Output = Message;

    fn create(link: AgentLink<Self>) -> Self {
        Self {
            link,
            subscribers: HashSet::new(),
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn handle_input(&mut self, msg: Self::Input, _id: HandlerId) {
        for sub in self.subscribers.iter() {
            self.link.respond(*sub, msg.clone());
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }
}

struct Handler {
    tournament_id: TournamentId,
    bracket_id: BracketId,
    dispatcher: Dispatcher<EventBus>,
    close_waker: Rc<Notify>,
}

impl EventHandler for Handler {
    fn dispatch(&mut self, msg: WebSocketMessage) {
        log::debug!("Received frame: {:?}", msg);

        match msg {
            WebSocketMessage::Bytes(buf) => {
                let mut buf = Cursor::new(buf);

                match Response::decode(&mut buf) {
                    Ok(resp) => self.dispatcher.send(Message::Response(resp)),
                    Err(err) => {
                        log::error!("Failed to decode websocket response: {}", err);
                    }
                }
            }
            WebSocketMessage::Text(_) => (),
            WebSocketMessage::Close => {
                self.close_waker.notify_all();

                self.dispatcher
                    .send(Message::Close(self.tournament_id, self.bracket_id));
            }
        }
    }
}
