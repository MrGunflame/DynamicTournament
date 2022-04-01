use std::rc::Rc;

use crate::api::tournament as api;
use crate::bracket_generator::{EntrantSpot, SingleElimination, Winner};
use crate::components::r#match::{Match, MatchMember};

use yew::prelude::*;

pub struct Bracket {
    state: SingleElimination<api::Team>,
}

impl Component for Bracket {
    type Message = Msg;
    type Properties = BracketProperties;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            state: SingleElimination::new(ctx.props().tournament.teams.clone()),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateWinner {
                index,
                winner_index,
            } => {
                self.state.update_winner(index, Winner::Team(winner_index));
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let rounds: Html = self
            .state
            .rounds_iter()
            .with_index()
            .map(|(round, starting_index)| {
                let matches: Html = round
                    .iter()
                    .enumerate()
                    .map(|(index, m)| {
                        let on_winner_update =
                            ctx.link()
                                .callback(move |winner_index: usize| Msg::UpdateWinner {
                                    index: starting_index + index,
                                    winner_index,
                                });

                        let teams = m.entrants.clone().map(|e| match e {
                            EntrantSpot::Entrant(team) => MatchMember::Entrant(team),
                            EntrantSpot::Empty => MatchMember::Placeholder("null".to_owned()),
                            EntrantSpot::TBD => MatchMember::Placeholder("TBD".to_owned()),
                        });

                        html! {
                            <Match teams={teams} on_winner_update={on_winner_update} />
                        }
                    })
                    .collect();

                html! {
                    <div class="bracket-round">
                        {matches}
                    </div>
                }
            })
            .collect();

        html! {
            <div class="bracket-matches">
                {rounds}
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct BracketProperties {
    pub tournament: Rc<api::Tournament>,
}

pub enum Msg {
    UpdateWinner { index: usize, winner_index: usize },
}
