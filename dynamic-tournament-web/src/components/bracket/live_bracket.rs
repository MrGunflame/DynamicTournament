use std::time::Duration;

use dynamic_tournament_api::v3::{
    id::{BracketId, TournamentId},
    tournaments::{
        brackets::{matches::Request, Bracket},
        entrants::Entrant,
        Tournament,
    },
};
use futures::StreamExt;
use futures::{channel::mpsc, SinkExt};
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html, Properties};
use yew_agent::{Bridge, Bridged};

use super::live_state::LiveState;
use super::Bracket as BracketComponent;
use crate::{api::Client, services::Message as WebSocketMessage};
use crate::{
    components::{
        movable_boxed::MovableBoxed,
        providers::{ClientProvider, Provider},
    },
    services::{EventBus, MessageLog, WebSocketService},
    utils::Rc,
};

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
    pub bracket: Rc<Bracket>,
    pub entrants: Rc<Vec<Entrant>>,
}

pub struct LiveBracket {
    websocket: Option<WebSocket>,
    _producer: Box<dyn Bridge<EventBus>>,

    // Ready state of the websocket. A `Some(WebSocket)` value only
    // means that the websocket exists, but it may not be connected yet.
    // Once `is_ready` is `true` the websocket is connected and ready for
    // use.
    is_ready: bool,
}

impl Component for LiveBracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut this = Self {
            websocket: None,
            _producer: EventBus::bridge(ctx.link().callback(Message::HandleMessage)),
            is_ready: false,
        };

        this.changed(ctx);
        this
    }

    // When the properties change we should close the existing socket and forget the existing
    // state and create a new one using the new properties.
    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        // self.state = None;
        let onready = ctx.link().callback(|_| Message::Ready);

        let client = ClientProvider::get(ctx);

        let websocket =
            match WebSocketService::new(&client, ctx.props().tournament.id, ctx.props().bracket.id)
            {
                Ok(mut websocket) => {
                    let (tx, mut rx) = mpsc::channel(32);

                    spawn_local(async move {
                        websocket.send(Request::SyncState).await;
                        onready.emit(());

                        while let Some(req) = rx.next().await {
                            websocket.send(req).await;
                        }
                    });

                    Some(WebSocket { tx })
                }
                Err(err) => {
                    MessageLog::error(err.to_string());
                    None
                }
            };

        self.websocket = websocket;
        self._producer = EventBus::bridge(ctx.link().callback(Message::HandleMessage));

        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Ready => {
                self.is_ready = true;

                true
            }
            Message::HandleMessage(msg) => match msg {
                WebSocketMessage::Response(_) => false,
                WebSocketMessage::Close => {
                    self.is_ready = false;
                    // Try to reconnect.
                    self.changed(ctx)
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tournament = ctx.props().tournament.clone();
        let bracket = ctx.props().bracket.clone();
        let entrants = ctx.props().entrants.clone();
        let websocket = self.websocket.clone();

        let is_live = self.is_ready;

        html! {
            <MovableBoxed>
                <LiveState {is_live} />
                <BracketComponent {tournament} {bracket} {entrants} {websocket} />
            </MovableBoxed>
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Ready,
    HandleMessage(WebSocketMessage),
}

#[derive(Clone, Debug)]
pub struct WebSocket {
    tx: mpsc::Sender<Request>,
}

impl WebSocket {
    pub fn new(
        client: &Client,
        tournament_id: TournamentId,
        bracket_id: BracketId,
    ) -> Option<Self> {
        let ws = match WebSocketService::new(&client, tournament_id, bracket_id) {
            Ok(ws) => ws,
            Err(err) => {
                log::error!("Failed to connect to server: {}", err);
                return None;
            }
        };

        let (tx, mut rx) = mpsc::channel(32);

        spawn_local(async move {
            loop {
                // Reconnect every 15s when disconnected.
                gloo_timers::future::sleep(Duration::new(15, 0)).await;
            }
        });

        Ok(Self { tx })
    }

    pub async fn send(&mut self, req: Request) {
        let _ = self.tx.send(req).await;
    }
}

impl PartialEq for WebSocket {
    fn eq(&self, other: &Self) -> bool {
        self.tx.same_receiver(&other.tx)
    }
}
