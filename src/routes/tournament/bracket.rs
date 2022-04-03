use std::rc::Rc;

use crate::api::tournament as api;
use crate::api::v1::tournament as api2;
use crate::bracket_generator::{EntrantSpot, EntrantWithScore, MatchWinner, SingleElimination};
use crate::components::config_provider::Config;
use crate::components::popup::Popup;
use crate::components::r#match::{Match, MatchMember};
use crate::components::update_bracket::BracketUpdate;

use yew::prelude::*;

pub struct Bracket {
    state: SingleElimination<EntrantWithScore<api::Team, u64>>,
    // Popup open for match with index.
    popup: Option<(usize, [MatchMember; 2])>,
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

        let state = match &ctx.props().bracket {
            Some(bracket) => SingleElimination::resume(bracket.0.clone()),
            None => SingleElimination::new(teams),
        };

        gloo_console::log!("created");
        gloo_console::log!(format!("{:?}", state));

        Self { state, popup: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateScore { index, scores } => {
                self.state.update_match(index, |m| {
                    m.entrants[0].unwrap_ref_mut().score = scores[0];
                    m.entrants[1].unwrap_ref_mut().score = scores[1];

                    for m in m.entrants.iter_mut() {
                        let m = m.unwrap_ref_mut();

                        // Team is the winner.
                        if m.score >= (ctx.props().tournament.best_of / 2) + 1 {
                            m.winner = true;

                            return Some(MatchWinner::Entrant(EntrantWithScore::new(
                                m.entrant.clone(),
                            )));
                        }
                    }

                    None
                });

                gloo_console::log!(format!("{:?}", self.state));

                let (config, _) = ctx
                    .link()
                    .context::<Config>(Callback::noop())
                    .expect("No ConfigProvider given");

                let tournament_id = ctx.props().tournament.id;

                let bracket = self.state.iter().cloned().collect();
                // Update server data.
                ctx.link().send_future_batch(async move {
                    let bracket = api2::Bracket(bracket);

                    match bracket.put(tournament_id, config).await {
                        Ok(_) => {
                            vec![Msg::UpdateScoreUI { index, scores }]
                        }
                        Err(err) => {
                            gloo_console::error!(format!("{:?}", err));
                            vec![]
                        }
                    }
                });

                false
            }
            Msg::UpdateScoreUI { index, scores } => {
                // Close the score update popup.
                self.popup = None;

                true
            }
            Msg::OpenUpdateMatchPopup { index, teams } => {
                self.popup = Some((index, teams));
                true
            }
            Msg::CloseUpdateMatchPopup => {
                self.popup = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        gloo_console::log!("render");

        let rounds: Html = self
            .state
            .rounds_iter()
            .with_index()
            .map(|(round, starting_index)| {
                let matches: Html = round
                    .iter()
                    .enumerate()
                    .map(|(index, m)| {
                        let teams = m.entrants.clone().map(|e| match e {
                            EntrantSpot::Entrant(team) => MatchMember::Entrant(team),
                            EntrantSpot::Empty => MatchMember::Placeholder("BYE".to_owned()),
                            EntrantSpot::TBD => MatchMember::Placeholder("TBD".to_owned()),
                        });

                        gloo_console::log!(format!("{}", index));

                        let teams2 = teams.clone();
                        let on_score_set =
                            ctx.link().callback(move |_| Msg::OpenUpdateMatchPopup {
                                index: starting_index + index,
                                teams: teams2.clone(),
                            });

                        html! {
                            <Match teams={teams} on_score_set={on_score_set} />
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

        let popup = match self.popup.clone() {
            Some((index, teams)) => {
                let m = self.state.get(index).unwrap();
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
                    .callback(move |scores| Msg::UpdateScore { index, scores });

                let on_close = ctx.link().callback(|_| Msg::CloseUpdateMatchPopup);

                html! {
                    <Popup on_close={on_close}>
                        <BracketUpdate teams={teams} scores={scores} on_submit={on_submit} />
                    </Popup>
                }
            }
            None => html! {},
        };

        html! {
            <>
                <div class="bracket-matches">
                    {rounds}
                </div>
                {popup}
            </>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct BracketProperties {
    pub tournament: Rc<api::Tournament>,
    pub bracket: Option<api2::Bracket>,
}

pub enum Msg {
    // Update score from the popup should update server data, the call Msg::UpdateScoreUI.
    UpdateScore {
        index: usize,
        scores: [u64; 2],
    },
    UpdateScoreUI {
        index: usize,
        scores: [u64; 2],
    },
    OpenUpdateMatchPopup {
        index: usize,
        teams: [MatchMember; 2],
    },
    CloseUpdateMatchPopup,
}
