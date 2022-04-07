use std::rc::Rc;

use yew::prelude::*;

use crate::api::tournament::{Team, Tournament};
use crate::api::v1::tournament::Bracket;

use crate::components::popup::Popup;
use crate::components::r#match::MatchMember;
use crate::components::update_bracket::BracketUpdate;

use super::BracketMatch;

use crate::bracket_generator::{
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

                    if m.entrants[0].unwrap_ref().score >= (ctx.props().tournament.best_of / 2) + 1
                    {
                        let winner = m.entrants[0].unwrap_ref_mut();
                        winner.winner = true;

                        let winner = m.entrants[0].unwrap_ref();
                        let looser = m.entrants[1].unwrap_ref();

                        return Some(MatchResult::Entrants {
                            winner: EntrantWithScore::new(winner.entrant.clone()),
                            looser: EntrantWithScore::new(looser.entrant.clone()),
                        });
                    }

                    if m.entrants[1].unwrap_ref().score >= (ctx.props().tournament.best_of / 2) + 1
                    {
                        let winner = m.entrants[1].unwrap_ref_mut();
                        winner.winner = true;

                        let winner = m.entrants[1].unwrap_ref();
                        let looser = m.entrants[0].unwrap_ref();

                        return Some(MatchResult::Entrants {
                            winner: EntrantWithScore::new(winner.entrant.clone()),
                            looser: EntrantWithScore::new(looser.entrant.clone()),
                        });
                    }

                    None
                });

                // TODO: HTTP PUT HERE TO SAVE NEW STATE
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

                let teams = [
                    match m.entrants[0] {
                        EntrantSpot::Entrant(ref e) => MatchMember::Entrant(e.clone()),
                        EntrantSpot::TBD => MatchMember::Placeholder("TBD".to_owned()),
                        EntrantSpot::Empty => MatchMember::Placeholder("BYE".to_owned()),
                    },
                    match m.entrants[1] {
                        EntrantSpot::Entrant(ref e) => MatchMember::Entrant(e.clone()),
                        EntrantSpot::TBD => MatchMember::Placeholder("TBD".to_owned()),
                        EntrantSpot::Empty => MatchMember::Placeholder("BYE".to_owned()),
                    },
                ];

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
                        <BracketUpdate teams={teams} scores={scores} on_submit={on_submit} />
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
                <div>
                    <span>{ "Winners Bracket" }</span>
                    <div class="bracket-matches">
                        {upper}
                    </div>
                </div>

                <div>
                    <span>{ "Grand Finals" }</span>
                    <div class="bracket-matches">
                        {finals}
                    </div>
                </div>
            </div>

            <div>
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
        Rc::ptr_eq(&self.tournament, &other.tournament) && self.bracket == other.bracket
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
}

pub enum Action {
    UpdateMatch,
}
