use gloo_storage::Storage;
use yew::callback::Callback;
use yew::prelude::*;

use super::team::Team;

use crate::api::tournament as api;
use crate::api::v1::auth::AuthCredentials;

use dynamic_tournament_generator::EntrantWithScore;

pub struct Match;

impl Component for Match {
    type Message = Msg;
    type Properties = MatchProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ScoreSet => {
                ctx.props().on_score_set.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let teams: Html = ctx
            .props()
            .teams
            .iter()
            .cloned()
            .map(|team| match team {
                MatchMember::Entrant(team) => {
                    let name = team.entrant.name.clone();
                    let score = team.score;

                    html! {<Team text={name} is_winner={team.winner} score={score} />}
                }
                MatchMember::Placeholder(s) => {
                    html! {
                        <Team text={s} is_winner={false} score={0} />
                    }
                }
            })
            .collect();

        // All spots must be filled for the button to become active.

        let credentials: gloo_storage::Result<AuthCredentials> =
            gloo_storage::LocalStorage::get("http_auth_data");

        let score_set_button = match credentials {
            Ok(_) => {
                if ctx.props().teams[0].is_entrant() && ctx.props().teams[1].is_entrant() {
                    let on_score_set = ctx.link().callback(|_| Msg::ScoreSet);

                    html! { <button onclick={on_score_set} disabled=false>{ "Set Score" }</button> }
                } else {
                    html! { <button title="Some entrant spots are not occupied." disabled=true>{ "Set Score" }</button> }
                }
            }
            Err(err) => {
                gloo_console::warn!(format!("Failed to read authorization credentials: {}", err));

                html! {
                    <button title="You are not logged in (or an error occured)." disabled=true>{ "Set Score" }</button>
                }
            }
        };

        html! {
            <div class="match">
                {teams}
                {score_set_button}
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct MatchProperties {
    pub teams: [MatchMember; 2],
    pub on_score_set: Callback<()>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MatchMember {
    Entrant(EntrantWithScore<api::Team, u64>),
    Placeholder(String),
}

impl MatchMember {
    pub fn is_entrant(&self) -> bool {
        matches!(self, Self::Entrant(_))
    }
}

pub enum Msg {
    // ScoreSet button
    ScoreSet,
}
