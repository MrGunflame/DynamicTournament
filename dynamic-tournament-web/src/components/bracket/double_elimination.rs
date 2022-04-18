use std::rc::Rc;

use yew::prelude::*;

use crate::components::popup::Popup;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::update_bracket::BracketUpdate;

use dynamic_tournament_api::tournament::{Bracket, Team, Tournament};

use super::{find_match_winner, Action, BracketMatch};

use dynamic_tournament_generator::{
    DoubleElimination, EntrantSpot, EntrantWithScore, Match, MatchResult,
};

pub struct DoubleEliminationBracket {
    state: DoubleElimination<EntrantWithScore<Team, u64>>,
    popup: Option<usize>,
}

impl Component for DoubleEliminationBracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let state = match &ctx.props().bracket {
            Some(bracket) => DoubleElimination::resume(bracket.0.clone()),
            _ => {
                let teams = ctx
                    .props()
                    .tournament
                    .teams
                    .iter()
                    .cloned()
                    .map(EntrantWithScore::new)
                    .collect();

                DoubleElimination::new(teams)
            }
        };

        Self { state, popup: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Action { index, action } => match action {
                Action::UpdateMatch => {
                    self.popup = Some(index);

                    true
                }
            },
            Message::ClosePopup => {
                self.popup = None;

                true
            }
            Message::UpdateScore { index, scores } => {
                self.state.update_match(index, |m| {
                    m.entrants[0].unwrap_ref_mut().score = scores[0];
                    m.entrants[1].unwrap_ref_mut().score = scores[1];

                    match find_match_winner(ctx.props().tournament.best_of, m) {
                        Some(index) => {
                            let winner = m.entrants[index].unwrap_ref_mut();
                            winner.winner = true;

                            let winner = m.entrants[index].unwrap_ref();
                            let looser = m.entrants[match index {
                                0 => 1,
                                1 => 0,
                                _ => unreachable!(),
                            }]
                            .unwrap_ref();

                            Some(MatchResult::Entrants {
                                winner: EntrantWithScore::new(winner.entrant.clone()),
                                looser: EntrantWithScore::new(looser.entrant.clone()),
                            })
                        }
                        _ => None,
                    }
                });

                let client = ClientProvider::take(ctx);

                let id = ctx.props().tournament.id;
                let bracket = Bracket(self.state.iter().cloned().collect());
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
            Message::UpdateScoreUI => {
                self.popup = None;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let popup = match self.popup {
            Some(index) => {
                // on_close handler for the popup.
                let on_close = ctx.link().callback(|_| Message::ClosePopup);

                let m = self.state.get(index).unwrap();

                let teams = m.entrants.clone();

                let scores = [
                    match m.entrants[0] {
                        EntrantSpot::Entrant(ref e) => e.score,
                        _ => 0,
                    },
                    match m.entrants[1] {
                        EntrantSpot::Entrant(ref e) => e.score,
                        _ => 0,
                    },
                ];

                let on_submit = ctx
                    .link()
                    .callback(move |scores| Message::UpdateScore { index, scores });

                html! {
                    <Popup on_close={on_close}>
                        <BracketUpdate {teams} scores={scores} on_submit={on_submit} />
                    </Popup>
                }
            }
            None => html! {},
        };

        let upper: Html = self
            .state
            .upper_bracket_iter()
            .with_index()
            .map(|(round, starting_index)| render_round(ctx, round, starting_index))
            .collect();

        let lower: Html = self
            .state
            .lower_bracket_iter()
            .with_index()
            .map(|(round, starting_index)| render_round(ctx, round, starting_index))
            .collect();

        let finals: Html = self
            .state
            .final_bracket_iter()
            .with_index()
            .map(|(m, index)| render_match(ctx, m, index))
            .collect();

        html! {
            <>
            <div class="flex-col">
                <div class="tourn-bracket">
                    <span>{ "Winners Bracket" }</span>
                    <div class="bracket-matches">
                        {upper}
                    </div>
                </div>

                <div class="bracket-flex-center tourn-bracket">
                    <div>
                        <span>{ "Grand Finals" }</span>
                        <div class="bracket-matches">
                            {finals}
                        </div>
                    </div>
                </div>
            </div>

            <div class="tourn-bracket">
                <span>{ "Losers Bracket" }</span>
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
    ctx: &Context<DoubleEliminationBracket>,
    round: &[Match<EntrantWithScore<Team, u64>>],
    starting_index: usize,
) -> Html {
    let round: Html = round
        .iter()
        .enumerate()
        .map(|(index, m)| {
            html! {render_match(ctx, m, starting_index + index)}
        })
        .collect();

    html! {
        <div class="bracket-round">
            {round}
        </div>
    }
}

fn render_match(
    ctx: &Context<DoubleEliminationBracket>,
    m: &Match<EntrantWithScore<Team, u64>>,
    index: usize,
) -> Html {
    let on_action = ctx
        .link()
        .callback(move |action| Message::Action { index, action });

    html! {
        <BracketMatch entrants={m.entrants.clone()} on_action={on_action} />
    }
}

pub enum Message {
    Action { index: usize, action: Action },
    ClosePopup,
    UpdateScore { index: usize, scores: [u64; 2] },
    UpdateScoreUI,
}
