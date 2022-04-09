use yew::callback::Callback;
use yew::prelude::*;

use super::team::Team;

use dynamic_tournament_api::tournament as api;
use dynamic_tournament_api::Client;
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

        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("No ClientProvider given");

        // All spots must be filled for the button to become active.

        let score_set_button = match client.is_authenticated() {
            true => {
                if ctx.props().teams[0].is_entrant() && ctx.props().teams[1].is_entrant() {
                    let on_score_set = ctx.link().callback(|_| Msg::ScoreSet);

                    html! { <button onclick={on_score_set} disabled=false>{ "Set Score" }</button> }
                } else {
                    html! { <button title="Some entrant spots are not occupied." disabled=true>{ "Set Score" }</button> }
                }
            }
            false => {
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

#[derive(Clone, Debug)]
pub enum MatchMember {
    Entrant(EntrantWithScore<api::Team, u64>),
    Placeholder(String),
}

impl MatchMember {
    pub fn is_entrant(&self) -> bool {
        matches!(self, Self::Entrant(_))
    }
}

impl PartialEq for MatchMember {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

pub enum Msg {
    // ScoreSet button
    ScoreSet,
}