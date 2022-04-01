use yew::callback::Callback;
use yew::prelude::*;

use super::team::Team;

use crate::api::tournament as api;
use crate::bracket_generator::EntrantWithScore;

pub struct Match;

impl Component for Match {
    type Message = Msg;
    type Properties = MatchProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(args) => {
                ctx.props().on_score_update.emit(args);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let teams: Html = ctx
            .props()
            .teams
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, team)| match team {
                MatchMember::Entrant(team) => {
                    let name = team.entrant.name.clone();
                    let score = team.score;


                    let cb = link.callback(move |score| {
                        Msg::Update(CallbackArgs {
                            team_index: i,
                            new_score: score + 1,
                        })
                    });

                    html! {<Team text={name} is_winner={team.winner} on_score_update={cb.clone()} score={score} />}
                }
                MatchMember::Placeholder(s) => {
                    let clos = Callback::from(|_: u64| {});

                    html! {
                        <Team text={s} is_winner={false} on_score_update={clos} score={0} />
                    }
                }
            })
            .collect();

        html! {
            <div class="match">
                {teams}
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct MatchProperties {
    pub teams: [MatchMember; 2],
    pub on_score_update: Callback<CallbackArgs>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MatchMember {
    Entrant(EntrantWithScore<api::Team, u64>),
    Placeholder(String),
}

pub struct CallbackArgs {
    pub team_index: usize,
    pub new_score: u64,
}

pub enum Msg {
    Update(CallbackArgs),
}
