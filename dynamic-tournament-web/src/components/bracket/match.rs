use yew::prelude::*;

use dynamic_tournament_api::tournament::Team;
use dynamic_tournament_api::Client;
use dynamic_tournament_generator::{EntrantSpot, EntrantWithScore};

use super::BracketTeam;

pub struct BracketMatch;

impl Component for BracketMatch {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateScore => {
                ctx.props().on_action.emit(Action::UpdateMatch);
            }
            Message::ResetMatch => (),
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let entrants: Html = ctx
            .props()
            .entrants
            .iter()
            .map(|entrant| {
                html! {
                    <BracketTeam entrant={entrant.clone()} />
                }
            })
            .collect();

        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("No ClientProvider given");

        let action_button = match client.is_authenticated() {
            true => {
                if ctx.props().entrants[0].is_entrant() && ctx.props().entrants[1].is_entrant() {
                    let onclick = ctx.link().callback(|_| Message::UpdateScore);

                    let on_reset = ctx.link().callback(|_| Message::ResetMatch);

                    html! {
                        <div class="match-action-buttons">
                            <button onclick={onclick} disabled=false>
                                <img src="/assets/pen-solid.svg" width="16px" height="16px" />
                            </button>
                            <button onclick={on_reset} disabled=false>
                                <img src="/assets/arrow-rotate-left-solid.svg" width="16px" height="16px" />
                            </button>
                        </div>
                    }
                } else {
                    html! {
                        <div class="match-action-buttons">
                            <button title="Some entrant spots are not occupied." disabled=true>
                                <img src="/assets/pen-solid.svg" width="16px" height="16px" />
                            </button>
                            <button title="Some entrant spots are not occupied." disabled=true>
                                <img src="/assets/arrow-rotate-left-solid.svg" width="16px" height="16px" />
                            </button>
                        </div>
                    }
                }
            }
            false => {
                html! {}
            }
        };

        html! {
            <div class="match">
                <div class="match-teams">
                    {entrants}
                </div>
                {action_button}
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub entrants: [EntrantSpot<EntrantWithScore<Team, u64>>; 2],
    pub on_action: Callback<Action>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

pub enum Message {
    UpdateScore,
    ResetMatch,
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    UpdateMatch,
}
