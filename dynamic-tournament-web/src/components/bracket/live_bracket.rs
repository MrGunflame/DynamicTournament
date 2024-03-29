use dynamic_tournament_api::v3::tournaments::{
    brackets::{matches::Request, Bracket},
    entrants::Entrant,
    Tournament,
};
use wasm_bindgen_futures::spawn_local;
use yew::{html, Component, Context, Html, Properties};
use yew_agent::{Bridge, Bridged};

use super::live_state::LiveState;
use super::Bracket as BracketComponent;
use crate::services::Message as WebSocketMessage;
use crate::{api::Action, components::Button};
use crate::{
    components::{
        movable_boxed::MovableBoxed,
        providers::{ClientProvider, Provider},
    },
    services::{EventBus, WebSocketService},
    utils::Rc,
};

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
    pub bracket: Rc<Bracket>,
    pub entrants: Rc<Vec<Entrant>>,
}

pub struct LiveBracket {
    websocket: Option<WebSocketService>,
    _producer: Box<dyn Bridge<EventBus>>,

    is_live: bool,
    panel: Panel,
}

impl Component for LiveBracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut this = Self {
            websocket: None,
            _producer: EventBus::bridge(ctx.link().callback(Message::Ws)),
            is_live: false,
            panel: Panel::default(),
        };

        // Watch for changes on API client and send a new authorization
        // message when the login becomes available/changes.
        // This will automatically refresh connections that live longer than
        // the lifetime of the token. It also automatically authorizes when the
        // token becomes available after the connection was already opened.
        let client = ClientProvider::get(ctx);
        let link = ctx.link().clone();
        ctx.link().send_future_batch(async move {
            loop {
                let action = client.changed().await;
                if matches!(action, Action::Login | Action::Refresh) {
                    let token = client.authorization().auth_token().unwrap().clone();
                    link.send_message(Message::Authorize(token.into_token()));
                }
            }
        });

        this.changed(ctx);
        this
    }

    // When the properties change we should close the existing socket and forget the existing
    // state and create a new one using the new properties.
    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.is_live = false;

        let client = ClientProvider::get(ctx);

        let websocket =
            WebSocketService::new(&client, ctx.props().tournament.id, ctx.props().bracket.id);

        self.websocket = Some(websocket);

        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Ws(msg) => match msg {
                WebSocketMessage::Response(_) => false,
                WebSocketMessage::Close(tournament_id, bracket_id) => {
                    if ctx.props().tournament.id == tournament_id
                        && ctx.props().bracket.id == bracket_id
                    {
                        self.is_live = false;
                        true
                    } else {
                        false
                    }
                }
                WebSocketMessage::Connect(tournament_id, bracket_id) => {
                    if ctx.props().tournament.id == tournament_id
                        && ctx.props().bracket.id == bracket_id
                    {
                        // Resync once connected
                        let mut ws = self.websocket.clone().unwrap();
                        spawn_local(async move {
                            let _ = ws.send(Request::SyncState).await;
                        });

                        self.is_live = true;
                        true
                    } else {
                        false
                    }
                }
            },
            Message::ChangePanel(panel) => {
                self.panel = panel;
                true
            }
            Message::Authorize(token) => {
                if let Some(ws) = &self.websocket {
                    let mut ws = ws.clone();

                    spawn_local(async move {
                        let _ = ws.send(Request::Authorize(token)).await;
                    });
                }

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tournament = ctx.props().tournament.clone();
        let bracket = ctx.props().bracket.clone();
        let entrants = ctx.props().entrants.clone();
        let websocket = self.websocket.clone();

        let is_live = self.is_live;

        let panel = self.panel;
        let on_panel_toggle = ctx.link().callback(move |_| {
            let panel = match panel {
                Panel::Matches => Panel::Standings,
                Panel::Standings => Panel::Matches,
            };

            Message::ChangePanel(panel)
        });

        let header = html! {
            <div>
                <Button onclick={on_panel_toggle} title="Toggle Standings">
                    <span>{ "Standings" }</span>
                </Button>
                <LiveState {is_live} />
            </div>
        };

        html! {
            <MovableBoxed {header}>
                <BracketComponent {tournament} {bracket} {entrants} {websocket} panel={self.panel} />
            </MovableBoxed>
        }
    }
}

pub enum Message {
    Ws(WebSocketMessage),
    ChangePanel(Panel),
    Authorize(String),
}

/// The currently displayed panel.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum Panel {
    #[default]
    Matches,
    Standings,
}
