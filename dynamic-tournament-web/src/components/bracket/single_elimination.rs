use std::rc::Rc;

use crate::components::popup::Popup;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::r#match::MatchMember;
use crate::components::update_bracket::BracketUpdate;

use dynamic_tournament_api::tournament::{Bracket, Team, Tournament};
use dynamic_tournament_generator::{
    EntrantSpot, EntrantWithScore, Match, MatchResult, SingleElimination,
};

use super::{Action, BracketMatch};

use yew::prelude::*;

pub struct SingleEliminationBracket {
    state: SingleElimination<EntrantWithScore<Team, u64>>,
    // Popup open for match with index.
    popup: Option<usize>,
}

impl Component for SingleEliminationBracket {
    type Message = Message;
    type Properties = BracketProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let teams = ctx
            .props()
            .tournament
            .teams
            .iter()
            .cloned()
            .map(EntrantWithScore::new)
            .collect();

        let state = match &ctx.props().bracket {
            Some(bracket) => SingleElimination::resume(bracket.0.clone()),
            None => SingleElimination::new(teams),
        };

        Self { state, popup: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Action { index, action: _ } => {
                self.popup = Some(index);

                true
            }
            Message::UpdateScore { index, scores } => {
                self.state.update_match(index, |m| {
                    m.entrants[0].unwrap_ref_mut().score = scores[0];
                    m.entrants[1].unwrap_ref_mut().score = scores[1];

                    if m.entrants[0].unwrap_ref().score > (ctx.props().tournament.best_of / 2) {
                        let winner = m.entrants[0].unwrap_ref_mut();
                        winner.winner = true;

                        let winner = m.entrants[0].unwrap_ref();
                        let looser = m.entrants[0].unwrap_ref();

                        return Some(MatchResult::Entrants {
                            winner: EntrantWithScore::new(winner.entrant.clone()),
                            looser: EntrantWithScore::new(looser.entrant.clone()),
                        });
                    }

                    if m.entrants[1].unwrap_ref().score > (ctx.props().tournament.best_of / 2) {
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

                let client = ClientProvider::take(ctx);

                let id = ctx.props().tournament.id;
                let bracket = Bracket(self.state.iter().cloned().collect());
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
            Message::UpdateScoreUI => {
                self.popup = None;

                true
            }
            Message::ClosePopup => {
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

        let bracket: Html = self
            .state
            .rounds_iter()
            .with_index()
            .map(|(round, starting_index)| render_round(ctx, round, starting_index))
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
    ctx: &Context<SingleEliminationBracket>,
    round: &[Match<EntrantWithScore<Team, u64>>],
    starting_index: usize,
) -> Html {
    let round: Html = round
        .iter()
        .enumerate()
        .map(|(index, m)| {
            html! {render_match(ctx,m,starting_index+index)}
        })
        .collect();

    html! {
        <div class="bracket-round">
            {round}
        </div>
    }
}

fn render_match(
    ctx: &Context<SingleEliminationBracket>,
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
