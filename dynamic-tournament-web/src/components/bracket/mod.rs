mod entrant;
mod live_bracket;
mod live_state;
mod r#match;
mod renderer;

use dynamic_tournament_api::v3::tournaments::brackets::matches::{
    ErrorResponse, Request, Response,
};
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use dynamic_tournament_core::options::TournamentOptionValues;
use dynamic_tournament_core::tournament::TournamentKind;
use dynamic_tournament_core::{EntrantScore, EntrantSpot, SingleElimination, System};
use entrant::BracketEntrant;
use r#match::{Action, BracketMatch};

use dynamic_tournament_core::tournament::Tournament;

use yew_agent::{Bridge, Bridged};

use yew::prelude::*;

use dynamic_tournament_api::v3::id::SystemId;
use dynamic_tournament_api::v3::tournaments::brackets::Bracket as ApiBracket;
use dynamic_tournament_api::v3::tournaments::Tournament as ApiTournament;

use crate::components::confirmation::Confirmation;
use crate::components::popup::Popup;
use crate::components::update_bracket::BracketUpdate;
use crate::services::errorlog::ErrorLog;
use crate::services::Message as WebSocketMessage;
use crate::services::{EventBus, WebSocketService};
use crate::utils::Rc;

use renderer::HtmlRenderer;

pub use live_bracket::LiveBracket;

use super::providers::{ClientProvider, Provider};

pub struct Bracket {
    _producer: Box<dyn Bridge<EventBus>>,
    popup: Option<PopupState>,
    state: Option<Tournament<String, EntrantScore<u64>>>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);
        let link = ctx.link().clone();
        ctx.link().send_future_batch(async move {
            loop {
                if client.changed().await.is_login() {
                    link.send_message(Message::Authorize);
                }
            }
        });

        Self {
            state: None,
            _producer: EventBus::bridge(ctx.link().callback(Message::HandleResponse)),
            popup: None,
        }
    }

    // Drop the existing state when changing to a new bracket.
    fn changed(&mut self, _ctx: &Context<Self>) -> bool {
        self.state = None;

        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::HandleResponse(resp) => {
                log::debug!("Received message: {:?}", resp);

                let resp = match resp {
                    WebSocketMessage::Response(resp) => resp,
                    // Close frames are not handled by this component.
                    WebSocketMessage::Close(_, _) => return false,
                    WebSocketMessage::Connect(_, _) => return false,
                };

                match resp {
                    Response::Error(err) => {
                        // The connection lagged bebing. Try to synchronize with the
                        // server again.
                        if err == ErrorResponse::Lagged {
                            log::debug!("Bracket is lagging");

                            let mut ws = ctx.props().websocket.clone().unwrap();
                            ctx.link().send_future_batch(async move {
                                let _ = ws.send(Request::SyncState).await;
                                vec![]
                            });
                        }
                        // We don't handle any other errors.

                        false
                    }

                    Response::UpdateMatch { index, nodes } => {
                        match &mut self.state {
                            Some(bracket) => {
                                bracket.update_match(index.try_into().unwrap(), |m, res| {
                                    let mut loser_index = None;

                                    for (i, (entrant, node)) in
                                        m.entrants.iter_mut().zip(nodes).enumerate()
                                    {
                                        if let EntrantSpot::Entrant(entrant) = entrant {
                                            entrant.data = node;
                                        }

                                        if node.winner {
                                            res.winner_default(entrant);
                                            loser_index = Some(match i {
                                                0 => 1,
                                                _ => 0,
                                            });
                                        }
                                    }

                                    if let Some(loser_index) = loser_index {
                                        res.loser_default(&m.entrants[loser_index]);
                                    }
                                });
                            }
                            // We have no data to update the bracket yet.
                            None => {
                                log::warn!("Received an UpdateMatch frame before initializing the state, ignoring");
                            }
                        }

                        true
                    }
                    Response::ResetMatch { index } => {
                        match &mut self.state {
                            Some(state) => {
                                state.update_match(index as usize, |_, res| {
                                    res.reset_default();
                                });
                            }
                            None => {
                                log::warn!("Received a ResetMatch frame before initializing the state, ignoring");
                            }
                        }

                        true
                    }
                    Response::SyncState(matches) => {
                        let system_kind = match ctx.props().bracket.system {
                            SystemId(1) => TournamentKind::SingleElimination,
                            SystemId(2) => TournamentKind::DoubleElimination,
                            _ => unimplemented!(),
                        };

                        let options = match system_kind {
                            TournamentKind::SingleElimination => ctx
                                .props()
                                .bracket
                                .options
                                .clone()
                                .merge(SingleElimination::<u8, EntrantScore<u8>>::options())
                                .unwrap(),
                            TournamentKind::DoubleElimination => TournamentOptionValues::default(),
                        };

                        let entrants = ctx
                            .props()
                            .bracket
                            .entrants
                            .iter()
                            .map(|id| {
                                // Map the EntrantId to an entrant name (from props).
                                for e in ctx.props().entrants.iter() {
                                    if e.id == *id {
                                        return match &e.inner {
                                            EntrantVariant::Player(player) => player.name.clone(),
                                            EntrantVariant::Team(team) => team.name.clone(),
                                        };
                                    }
                                }

                                // Id was not found in entrants.
                                String::from("Unknown")
                            })
                            .collect();

                        self.state =
                            match Tournament::resume(system_kind, entrants, matches, options) {
                                Ok(tournament) => Some(tournament),
                                Err(err) => {
                                    ErrorLog::error(err.to_string());
                                    None
                                }
                            };

                        true
                    }
                    _ => false,
                }
            }
            Message::Action { index, action } => {
                log::debug!("Called action {:?} on {}", action, index);

                match action {
                    Action::UpdateMatch => {
                        self.popup = Some(PopupState::UpdateScores(index));
                    }
                    Action::ResetMatch => {
                        self.popup = Some(PopupState::ResetMatch(index));
                    }
                }

                true
            }

            Message::ClosePopup => {
                self.popup = None;
                true
            }
            Message::UpdateMatch { index, nodes } => {
                if let Some(websocket) = &ctx.props().websocket {
                    let mut websocket = websocket.clone();

                    ctx.link().send_future_batch(async move {
                        let _ = websocket
                            .send(Request::UpdateMatch {
                                index: index.try_into().unwrap(),
                                nodes,
                            })
                            .await;

                        vec![Message::ClosePopup]
                    });
                }

                false
            }
            Message::ResetMatch(index) => {
                if let Some(websocket) = &ctx.props().websocket {
                    let mut websocket = websocket.clone();
                    ctx.link().send_future_batch(async move {
                        let _ = websocket
                            .send(Request::ResetMatch {
                                index: index.try_into().unwrap(),
                            })
                            .await;

                        vec![Message::ClosePopup]
                    });
                }

                false
            }
            Message::Authorize => {
                if let Some(websocket) = &ctx.props().websocket {
                    // Return early if there is not token set.
                    let auth = ClientProvider::get(ctx).authorization();
                    let token = match auth.auth_token().cloned() {
                        Some(token) => token,
                        None => return false,
                    };

                    let mut websocket = websocket.clone();
                    ctx.link().send_future_batch(async move {
                        let _ = websocket.send(Request::Authorize(token.to_string())).await;
                        vec![]
                    });
                }

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if let Some(bracket) = &self.state {
            let popup = match self.popup {
                Some(PopupState::UpdateScores(index)) => {
                    let on_close = ctx.link().callback(|_| Message::ClosePopup);

                    let m = &bracket.matches()[index];

                    let entrants = m
                        .entrants
                        .map(|e| e.map(|e| e.entrant(bracket).unwrap().clone()));

                    let nodes = m.entrants.map(|e| e.unwrap().data);

                    let on_submit = ctx
                        .link()
                        .callback(move |nodes| Message::UpdateMatch { index, nodes });

                    html! {
                        <Popup on_close={on_close}>
                            <BracketUpdate teams={entrants} {nodes} on_submit={on_submit} />
                        </Popup>
                    }
                }
                Some(PopupState::ResetMatch(index)) => {
                    let on_close = ctx.link().callback(|_| Message::ClosePopup);

                    let on_confirm = ctx.link().callback(move |_| Message::ResetMatch(index));

                    html! {
                        <Confirmation {on_close} {on_confirm}>
                            <span>{ "Are you sure to reset this match? This will also reset matches depending on the result of this match." }</span>
                        </Confirmation>
                    }
                }
                None => html! {},
            };

            let bracket = HtmlRenderer::new(bracket, ctx).into_output();

            html! {
                <>
                    { bracket }
                    { popup }
                </>
            }
        } else {
            html! { <span>{ "Loading" }</span> }
        }
    }
}

pub enum Message {
    /// Authorize using the current credentials. This may be sent be multiple times.
    Authorize,
    HandleResponse(WebSocketMessage),
    Action {
        index: usize,
        action: Action,
    },
    ClosePopup,
    UpdateMatch {
        index: usize,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch(usize),
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Properties {
    pub tournament: Rc<ApiTournament>,
    pub bracket: Rc<ApiBracket>,
    pub entrants: Rc<Vec<Entrant>>,
    pub websocket: Option<WebSocketService>,
}

enum PopupState {
    UpdateScores(usize),
    ResetMatch(usize),
}
