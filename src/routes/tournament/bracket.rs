use std::rc::Rc;

use crate::api::tournament as api;
use crate::bracket_generator::{EntrantSpot, EntrantWithScore, SingleElimination, Winner};
use crate::components::r#match::{CallbackArgs, Match, MatchMember};

use yew::prelude::*;

pub struct Bracket {
    state: SingleElimination<EntrantWithScore<api::Team, u64>>,
}

impl Component for Bracket {
    type Message = Msg;
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

        Self {
            state: SingleElimination::new(teams),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateEntrant {
                index,
                team_index,
                new_score,
            } => {
                // Target index of the winning team updated.
                let target_index = self.state.match_index(index) % 2;

                let m = self.state.get_mut(index).unwrap();
                m.entrants[team_index].unwrap_ref_mut().score = new_score;

                if new_score >= (ctx.props().tournament.best_of / 2) + 1 {
                    m.entrants[team_index].unwrap_ref_mut().winner = true;

                    self.state
                        .update_winner_callback(index, Winner::Team(team_index), |m| {
                            m.entrants[target_index].unwrap_ref_mut().score = 0;
                            m.entrants[target_index].unwrap_ref_mut().winner = false;
                        });
                }

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
                        let on_score_update =
                            ctx.link()
                                .callback(move |args: CallbackArgs| Msg::UpdateEntrant {
                                    index: starting_index + index,
                                    team_index: args.team_index,
                                    new_score: args.new_score,
                                });

                        let teams = m.entrants.clone().map(|e| match e {
                            EntrantSpot::Entrant(team) => MatchMember::Entrant(team),
                            EntrantSpot::Empty => MatchMember::Placeholder("null".to_owned()),
                            EntrantSpot::TBD => MatchMember::Placeholder("TBD".to_owned()),
                        });

                        // gloo_console::log!(format!("{:?}", teams));

                        html! {
                            <Match teams={teams} on_score_update={on_score_update} />
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
    UpdateEntrant {
        index: usize,
        team_index: usize,
        new_score: u64,
    },
}
