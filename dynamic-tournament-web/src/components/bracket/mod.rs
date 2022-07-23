mod entrant;
mod r#match;
mod renderer;

use dynamic_tournament_api::v3::tournaments::brackets::matches::Frame;
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use dynamic_tournament_core::options::TournamentOptionValues;
use dynamic_tournament_core::tournament::TournamentKind;
use dynamic_tournament_core::{EntrantScore, EntrantSpot, SingleElimination, System};
use entrant::BracketEntrant;
use r#match::{Action, BracketMatch};

use dynamic_tournament_core::tournament::Tournament;

use yew_agent::{Bridge, Bridged};

use std::rc::Rc;

use yew::prelude::*;

use dynamic_tournament_api::v3::id::SystemId;
use dynamic_tournament_api::v3::tournaments::brackets::Bracket as ApiBracket;
use dynamic_tournament_api::v3::tournaments::Tournament as ApiTournament;

use crate::components::confirmation::Confirmation;
use crate::components::popup::Popup;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::update_bracket::BracketUpdate;
use crate::services::errorlog::ErrorLog;
use crate::services::{EventBus, MessageLog, WebSocketService};

use renderer::HtmlRenderer;

pub struct Bracket {
    websocket: Option<WebSocketService>,
    _producer: Box<dyn Bridge<EventBus>>,
    popup: Option<PopupState>,
    state: Option<Tournament<String, EntrantScore<u64>>>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let websocket =
            match WebSocketService::new(&client, ctx.props().tournament.id, ctx.props().bracket.id)
            {
                Ok(websocket) => {
                    let mut ws = websocket.clone();
                    ctx.link().send_future_batch(async move {
                        ws.send(Frame::SyncMatchesRequest).await;

                        vec![]
                    });

                    Some(websocket)
                }
                Err(err) => {
                    MessageLog::error(err.to_string());
                    None
                }
            };

        Self {
            state: None,
            websocket,
            _producer: EventBus::bridge(ctx.link().callback(Message::HandleFrame)),
            popup: None,
        }
    }

    // When the properties change we should close the existing socket and forget the existing
    // state and create a new one using the new properties.
    fn changed(&mut self, ctx: &Context<Self>) -> bool {
        self.state = None;

        let client = ClientProvider::get(ctx);

        let websocket =
            match WebSocketService::new(&client, ctx.props().tournament.id, ctx.props().bracket.id)
            {
                Ok(websocket) => {
                    let mut ws = websocket.clone();
                    ctx.link().send_future_batch(async move {
                        ws.send(Frame::SyncMatchesRequest).await;

                        vec![]
                    });

                    Some(websocket)
                }
                Err(err) => {
                    MessageLog::error(err.to_string());
                    None
                }
            };

        self.websocket = websocket;
        self._producer = EventBus::bridge(ctx.link().callback(Message::HandleFrame));

        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::HandleFrame(msg) => {
                log::debug!("Received message: {:?}", msg);

                match msg {
                    Frame::UpdateMatch { index, nodes } => {
                        log::warn!(
                            "Received an UpdateMatch frame before initializing the state, ignoring"
                        );

                        let bracket = self.state.as_mut().unwrap();

                        {
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
                                            _ => 1,
                                        });
                                    }
                                }

                                if let Some(loser_index) = loser_index {
                                    res.loser_default(&m.entrants[loser_index]);
                                }
                            });
                        }
                    }
                    Frame::ResetMatch { index } => {
                        log::warn!(
                            "Received a ResetMatch frame before initializing the state, ignoring"
                        );

                        let bracket = self.state.as_mut().unwrap();

                        {
                            bracket.update_match(index, |_, res| {
                                res.reset_default();
                            });
                        }
                    }
                    Frame::SyncMatchesResponse(matches) => {
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
                    }
                    _ => (),
                }

                true
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
                if let Some(websocket) = &self.websocket {
                    let mut websocket = websocket.clone();

                    ctx.link().send_future_batch(async move {
                        websocket
                            .send(Frame::UpdateMatch {
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
                if let Some(websocket) = &self.websocket {
                    let mut websocket = websocket.clone();
                    ctx.link().send_future_batch(async move {
                        websocket.send(Frame::ResetMatch { index }).await;

                        vec![Message::ClosePopup]
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
    HandleFrame(Frame),
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

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub tournament: Rc<ApiTournament>,
    pub bracket: Rc<ApiBracket>,
    pub entrants: Rc<Vec<Entrant>>,
}

impl PartialEq for Properties {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
            && Rc::ptr_eq(&self.bracket, &other.bracket)
            && Rc::ptr_eq(&self.entrants, &other.entrants)
    }
}

enum PopupState {
    UpdateScores(usize),
    ResetMatch(usize),
}
