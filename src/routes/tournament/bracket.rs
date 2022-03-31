use std::rc::Rc;

use crate::api::tournament as api;
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
        let teams: Vec<Option<api::Team>> = ctx
            .props()
            .tournament
            .teams
            .clone()
            .into_iter()
            .map(|t| Some(t))
            .collect();

        // Number of matches wanted.
        let num_wanted = numer_teams_wanted(teams.len());

        // Starting inserting null teams in every uneven index.
        // let mut i = 1;
        // while i < num_wanted - teams.len() {
        //     teams.insert(i, None);

        //     i += 2;
        // }

        let mut i = 0;
        while i < teams.len() {
            let team1 = MatchMember::Entrant(teams.get(i).unwrap().clone().unwrap());
            let team2 = match teams.get(i + 1) {
                Some(Some(team)) => MatchMember::Entrant(team.clone()),
                _ => MatchMember::Placeholder("null".to_string()),
            };

            initial_round.push([team1, team2]);

            i += 2;
        }

        let mut i = 0;
        while initial_round.len() < num_wanted / 2 {
            let teams = initial_round.get_mut(i).unwrap();

            let fill_match = [
                teams[1].clone(),
                MatchMember::Placeholder("null".to_owned()),
            ];

            teams[1] = MatchMember::Placeholder("null".to_owned());

            initial_round.push(fill_match);

            i += 1;
        }

        gloo_console::log!(format!("{}", num_wanted));

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

                // self.state.bracket.update_match(
                //     round_index,
                //     match_index,
                //     match winner_index {
                //         0 => Winner::Team1,
                //         1 => Winner::Team2,
                //         w => panic!("Got winner_index {} in a 2 team match", w),
                //     },
                //     |m| {},
                // );

                // true
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

        // Starts with the inital number of matches, then halfs it until 1 is reached.
        // let mut num_matches = ctx.props().tournament.teams.len() / 2;
        // let mut buffer = 0;
        // while num_matches > 1 {
        //     for i in 0..num_matches {
        //         buffer += 1;
        //     }

        //     num_matches /= 2;
        // }

        // let bracket = &self.state.bracket;
        // let rounds: Html = bracket
        //     .rounds_iter()
        //     .enumerate()
        //     .map(|(round_index, round)| {
        //         let matches: Html = round
        //             .iter()
        //             .enumerate()
        //             .map(|(match_index, m)| {
        //                 let on_winner_update =
        //                     ctx.link()
        //                         .callback(move |winner_index: usize| Msg::UpdateWinner {
        //                             round_index,
        //                             match_index,
        //                             winner_index,
        //                         });

        //                 let teams = m.entrants.clone().map(|e| match e {
        //                     EntrantSpot::Entrant(team) => MatchMember::Entrant(team),
        //                     EntrantSpot::Empty => MatchMember::Placeholder("null".to_owned()),
        //                     EntrantSpot::TBD => MatchMember::Placeholder("TBD".to_owned()),
        //                 });

        //                 html! {
        //                     <Match teams={teams} on_winner_update={on_winner_update} />
        //                 }
        //             })
        //             .collect();

        //         html! {
        //             <div class="bracket-round">
        //                 {matches}
        //             </div>
        //         }
        //     })
        //     .collect();

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

fn numer_teams_wanted(teams: usize) -> usize {
    let mut i = 1;
    while i < teams {
        i = i << 1;
    }

    i
}
