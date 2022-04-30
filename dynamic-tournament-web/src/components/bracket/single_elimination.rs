use std::ops::DerefMut;
use std::rc::Rc;

use crate::components::confirmation::Confirmation;
use crate::components::popup::Popup;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::update_bracket::BracketUpdate;

use dynamic_tournament_api::tournament::{Bracket, Team, Tournament};
use dynamic_tournament_generator::{Entrant, EntrantScore, EntrantSpot, Match, SingleElimination};

use super::{Action, BracketMatch};

use yew::prelude::*;

/// A bracket for a [`SingleElimination`] tournament.
pub struct SingleEliminationBracket {
    state: SingleElimination<Team, EntrantScore<u64>>,
    // Popup open for match with index.
    popup: Option<PopupState>,
}

impl Component for SingleEliminationBracket {
    type Message = Message;
    type Properties = BracketProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let teams = ctx.props().tournament.teams.clone();

        let state = match &ctx.props().bracket {
            Some(bracket) => SingleElimination::resume(teams.into(), bracket.0.clone()).unwrap(),
            _ => SingleElimination::new(teams.into_iter()),
        };

        Self { state, popup: None }
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
            Message::UpdateScoreUI => {
                self.popup = None;

                true
            }
            Message::ClosePopup => {
                self.popup = None;

                true
            }
            Message::ResetMatch(index) => {
                self.state.update_match(index, |_, res| {
                    res.reset_default();
                });

                let client = ClientProvider::take(ctx);

                let id = ctx.props().tournament.id;
                let bracket = Bracket(self.state.matches().clone());
                // Update server data.
                ctx.link().send_future_batch(async move {
                    let client = client.tournaments();
                    let client = client.bracket(id);

                    match client.put(&bracket).await {
                        Ok(_) => vec![Message::UpdateScoreUI],
                        Err(err) => {
                            log::error!("{}", err);
                            vec![]
                        }
                    }
                });

                false
            }
            Message::UpdateMatch { index, nodes } => {
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

                let client = ClientProvider::take(ctx);

                let id = ctx.props().tournament.id;
                let bracket = Bracket(self.state.matches().clone());
                // Update server data.
                ctx.link().send_future_batch(async move {
                    let client = client.tournaments();
                    let client = client.bracket(id);

                    match client.put(&bracket).await {
                        Ok(_) => vec![Message::UpdateScoreUI],
                        Err(err) => {
                            log::error!("{}", err);
                            vec![]
                        }
                    }
                });

                false
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

        let bracket: Html = self
            .state
            .rounds_iter()
            .with_index()
            .map(|(starting_index, round)| render_round(&self.state, ctx, round, starting_index))
            .collect();

        html! {
            <>
                <div class="bracket-matches">
                    {bracket}
                </div>
                {popup}
            </>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct BracketProperties {
    pub tournament: Rc<Tournament>,
    pub bracket: Option<Rc<Bracket>>,
}

impl PartialEq for BracketProperties {
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
    state: &SingleElimination<Team, EntrantScore<u64>>,
    ctx: &Context<SingleEliminationBracket>,
    round: &[Match<Entrant<EntrantScore<u64>>>],
    starting_index: usize,
) -> Html {
    let round: Html = round
        .iter()
        .enumerate()
        .map(|(index, m)| {
            html! {render_match(state, ctx,m,starting_index+index,index)}
        })
        .collect();

    html! {
        <div class="bracket-round">
            {round}
        </div>
    }
}

fn render_match(
    state: &SingleElimination<Team, EntrantScore<u64>>,
    ctx: &Context<SingleEliminationBracket>,
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
        <BracketMatch {entrants} {nodes} on_action={on_action} number={match_index+1} />
    }
}

pub enum Message {
    Action {
        index: usize,
        action: Action,
    },
    ClosePopup,
    UpdateScoreUI,
    ResetMatch(usize),
    UpdateMatch {
        index: usize,
        nodes: [EntrantScore<u64>; 2],
    },
}

/// The popup can either be the update-scores dialog or the reset match confirmation.
pub enum PopupState {
    UpdateScores(usize),
    ResetMatch(usize),
}
