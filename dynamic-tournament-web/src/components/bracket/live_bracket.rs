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
}

impl Component for LiveBracket {
    type Message = WebSocketMessage;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let mut this = Self {
            websocket: None,
            _producer: EventBus::bridge(ctx.link().callback(|msg| msg)),
            is_live: false,
        };

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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tournament = ctx.props().tournament.clone();
        let bracket = ctx.props().bracket.clone();
        let entrants = ctx.props().entrants.clone();
        let websocket = self.websocket.clone();

        let is_live = self.is_live;

        let header = html! { <LiveState {is_live} /> };

        html! {
            <MovableBoxed {header}>
                <BracketComponent {tournament} {bracket} {entrants} {websocket} />
            </MovableBoxed>
        }
    }
}
