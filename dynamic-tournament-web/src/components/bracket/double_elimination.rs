use std::ops::DerefMut;
use std::rc::Rc;

use dynamic_tournament_api::websocket;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use crate::components::confirmation::Confirmation;
use crate::components::popup::Popup;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::update_bracket::BracketUpdate;
use crate::services::{EventBus, WebSocketService};

use dynamic_tournament_api::tournament::{Bracket, Team, Tournament};

use super::{Action, BracketMatch};

use dynamic_tournament_generator::{DoubleElimination, Entrant, EntrantScore, EntrantSpot, Match};

/// A bracket for a [`DoubleElimination`] tournament.
pub struct DoubleEliminationBracket {
    state: DoubleElimination<Team, EntrantScore<u64>>,
    popup: Option<PopupState>,
    websocket: WebSocketService,
    _producer: Box<dyn Bridge<EventBus>>,
}

impl Component for DoubleEliminationBracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let teams = ctx.props().tournament.entrants.clone().unwrap_teams();

        let state = match &ctx.props().bracket {
            // Some(bracket) => DoubleElimination::resume(bracket.0.clone()).unwrap(),
            _ => DoubleElimination::new(teams.into_iter()),
        };

        let client = ClientProvider::take(ctx);
        let websocket = WebSocketService::new(client, ctx.props().tournament.id.0);

        Self {
            state,
            popup: None,
            websocket,
            _producer: EventBus::bridge(ctx.link().callback(Message::HandleMessage)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Action { index, action } => match action {
                Action::UpdateMatch => {
                    self.popup = Some(PopupState::UpdateScores(index));

                    true
                }
                Action::ResetMatch => {
                    self.popup = Some(PopupState::ResetMatch(index));

                    true
                }
            },
            Message::ClosePopup => {
                self.popup = None;

                true
            }
            Message::UpdateMatch { index, nodes } => {
                let mut websocket = self.websocket.clone();
                ctx.link().send_future_batch(async move {
                    websocket
                        .send(dynamic_tournament_api::websocket::Message::UpdateMatch {
                            index: index.try_into().unwrap(),
                            nodes,
                        })
                        .await;

                    vec![Message::ClosePopup]
                });

                false
            }
            Message::ResetMatch(index) => {
                let mut websocket = self.websocket.clone();
                ctx.link().send_future_batch(async move {
                    websocket
                        .send(dynamic_tournament_api::websocket::Message::ResetMatch { index })
                        .await;

                    vec![Message::ClosePopup]
                });

                false
            }
            Message::HandleMessage(msg) => {
                log::debug!("Got message: {:?}", msg);
                match msg {
                    websocket::Message::UpdateMatch { index, nodes } => {
                        let index = index.try_into().unwrap();

                        self.state.update_match(index, |m, res| {
                            let mut has_winner = false;

                            for (entrant, node) in m.entrants.iter_mut().zip(nodes.into_iter()) {
                                if let EntrantSpot::Entrant(entrant) = entrant {
                                    *entrant.deref_mut() = node;
                                }

                                if node.winner {
                                    res.winner_default(entrant);
                                    has_winner = true;
                                    continue;
                                }

                                if has_winner {
                                    res.loser_default(entrant);
                                    break;
                                }
                            }
                        });
                    }
                    websocket::Message::ResetMatch { index } => {
                        self.state.update_match(index, |_, res| {
                            res.reset_default();
                        });
                    }
                    _ => (),
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let popup = match self.popup {
            Some(PopupState::UpdateScores(index)) => {
                // on_close handler for the popup.
                let on_close = ctx.link().callback(|_| Message::ClosePopup);

                let m = self.state.matches().get(index).unwrap();

                let teams = m
                    .entrants
                    .clone()
                    .map(|e| e.map(|e| e.entrant(&self.state).clone()));

                let nodes = m.entrants.clone().map(|e| e.unwrap().data);

                let on_submit = ctx
                    .link()
                    .callback(move |nodes| Message::UpdateMatch { index, nodes });

                html! {
                    <Popup on_close={on_close}>
                        <BracketUpdate {teams} {nodes} on_submit={on_submit} />
                    </Popup>
                }
            }
            Some(PopupState::ResetMatch(index)) => {
                let on_close = ctx.link().callback(|_| Message::ClosePopup);

                let on_confirm = ctx.link().callback(move |_| Message::ResetMatch(index));

                html! {
                    <Confirmation {on_close} {on_confirm} />
                }
            }
            None => html! {},
        };

        let upper: Html = self
            .state
            .upper_bracket_iter()
            .with_index()
            .map(|(starting_index, round)| render_round(&self.state, ctx, round, starting_index))
            .collect();

        let lower: Html = self
            .state
            .lower_bracket_iter()
            .with_index()
            .map(|(starting_index, round)| render_round(&self.state, ctx, round, starting_index))
            .collect();

        let finals: Html = self
            .state
            .final_bracket_iter()
            .with_index()
            .map(|(index, m)| render_round(&self.state, ctx, m, index))
            .collect();

        html! {
            <>
            <div class="flex-col">
                <div class="tourn-bracket">
                    <span class="title-label">{ "Winners Bracket" }</span>
                    <div class="bracket-matches">
                        {upper}
                    </div>
                </div>

                <div class="bracket-flex-center tourn-bracket">
                    <div>
                        <span class="title-label">{ "Grand Finals" }</span>
                        <div class="bracket-matches">
                            {finals}
                        </div>
                    </div>
                </div>
            </div>

            <div class="tourn-bracket">
                <span class="title-label">{ "Losers Bracket" }</span>
                <div class="bracket-matches">
                    {lower}
                </div>
            </div>

            {popup}
            </>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
    pub bracket: Option<Rc<Bracket>>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
            && self
                .bracket
                .as_ref()
                .zip(other.bracket.as_ref())
                .map_or(false, |(a, b)| Rc::ptr_eq(a, b))
    }
}

fn render_round(
    state: &DoubleElimination<Team, EntrantScore<u64>>,
    ctx: &Context<DoubleEliminationBracket>,
    round: &[Match<Entrant<EntrantScore<u64>>>],
    starting_index: usize,
) -> Html {
    let round: Html = round
        .iter()
        .enumerate()
        .map(|(index, m)| {
            html! {render_match(state, ctx, m, starting_index + index, index)}
        })
        .collect();

    html! {
        <div class="bracket-round">
            {round}
        </div>
    }
}

fn render_match(
    state: &DoubleElimination<Team, EntrantScore<u64>>,
    ctx: &Context<DoubleEliminationBracket>,
    m: &Match<Entrant<EntrantScore<u64>>>,
    index: usize,
    match_index: usize,
) -> Html {
    let on_action = ctx
        .link()
        .callback(move |action| Message::Action { index, action });

    let entrants = m
        .entrants
        .clone()
        .map(|e| e.map(|e| e.entrant(state).clone()));

    let nodes = m.entrants.clone().map(|e| e.map(|e| e.data));

    html! {
        <BracketMatch {entrants} {nodes} on_action={on_action} number={match_index + 1} />
    }
}

pub enum Message {
    Action {
        index: usize,
        action: Action,
    },
    ClosePopup,
    ResetMatch(usize),
    UpdateMatch {
        index: usize,
        nodes: [EntrantScore<u64>; 2],
    },
    HandleMessage(websocket::Message),
}

/// The popup can either be the update-scores dialog or the reset match confirmation.
pub enum PopupState {
    UpdateScores(usize),
    ResetMatch(usize),
}
