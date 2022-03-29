use std::rc::Rc;

use crate::components::r#match::{Match, MatchMember};

use yew::prelude::*;

pub struct Bracket {
    state: BracketState,
}

impl Component for Bracket {
    type Message = Msg;
    type Properties = BracketProperties;

    fn create(ctx: &Context<Self>) -> Self {
        let mut initial_round = Vec::with_capacity(ctx.props().tournament.teams.len() / 2);
        let mut i = 0;
        while i < ctx.props().tournament.teams.len() {
            let team1 = MatchMember::Entrant(ctx.props().tournament.teams.get(i).unwrap().clone());
            let team2 = match ctx.props().tournament.teams.get(i + 1) {
                Some(team) => MatchMember::Entrant(team.clone()),
                None => MatchMember::Placeholder("null".to_string()),
            };

            initial_round.push([team1, team2]);

            i += 2;
        }

        let mut initial_rounds = vec![initial_round];
        while initial_rounds.last().unwrap().len() > 1 {
            let mut matches = Vec::new();
            for _ in 0..initial_rounds.last().unwrap().len() / 2 {
                matches.push([
                    MatchMember::Placeholder("TBD".to_string()),
                    MatchMember::Placeholder("TBD".to_string()),
                ]);
            }

            initial_rounds.push(matches);
        }

        let state = BracketState {
            rounds: initial_rounds,
        };

        Self { state }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateWinner {
                round_index,
                match_index,
                winner_index,
            } => {
                // Skip next round on finals.
                if round_index < self.state.rounds.len() - 1 {
                    self.state.rounds[round_index + 1][match_index >> 1][match_index % 2] =
                        self.state.rounds[round_index][match_index][winner_index].clone();

                    true
                } else {
                    false
                }
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let rounds: Html = self
            .state
            .rounds
            .iter()
            .enumerate()
            .map(|(round_index, round)| {
                let matches: Html = round
                    .iter()
                    .enumerate()
                    .map(|(match_index, m)| {
                        let on_winner_update =
                            ctx.link()
                                .callback(move |winner_index: usize| Msg::UpdateWinner {
                                    round_index,
                                    match_index,
                                    winner_index,
                                });

                        html! {
                            <Match teams={m.clone()} on_winner_update={on_winner_update} />
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
    pub tournament: Rc<crate::MatchmakerInput>,
}

struct BracketState {
    rounds: Vec<Vec<[MatchMember; 2]>>,
}

pub enum Msg {
    UpdateWinner {
        round_index: usize,
        match_index: usize,
        winner_index: usize,
    },
}
