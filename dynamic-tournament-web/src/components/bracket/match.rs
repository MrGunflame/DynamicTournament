use yew::prelude::*;

use dynamic_tournament_api::tournament::Team;
use dynamic_tournament_generator::{EntrantSpot, EntrantWithScore};

use super::BracketTeam;
use crate::components::button::Button;
use crate::components::providers::{ClientProvider, Provider};

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

        let client = ClientProvider::take(ctx);

        let action_button = match client.is_authenticated() {
            true => {
                if ctx.props().entrants[0].is_entrant() && ctx.props().entrants[1].is_entrant() {
                    let onclick = ctx.link().callback(|_| Message::UpdateScore);

                    let on_reset = ctx.link().callback(|_| Message::ResetMatch);

                    html! {
                        <div class="match-action-buttons">
                            <Button classes="" {onclick} title="Edit">
                                <i aria-hidden="true" class="fa-solid fa-pen fa-xl"></i>
                                <span class="sr-only">{ "Edit" }</span>
                            </Button>
                            <Button classes="" onclick={on_reset} title="Reset">
                                <i aria-hidden="true" class="fa-solid fa-rotate-left fa-xl"></i>
                                <span class="sr-only">{ "Reset" }</span>
                            </Button>
                        </div>
                    }
                } else {
                    html! {
                        <div class="match-action-buttons">
                            <Button classes="" title="Edit (Some entrant spots are not occupied.)" disabled=true>
                                <i aria-hidden="true" class="fa-solid fa-pen fa-xl"></i>
                                <span class="sr-only">{ "Edit (Some entrant spots are not occupied.)" }</span>
                            </Button>
                            <Button classes="" title="Reset (Some entrant spots are not occupied.)" disabled=true>
                                <i aria-hidden="true" class="fa-solid fa-rotate-left fa-xl"></i>
                                <span class="sr-only">{ "Reset (Some entrant spots are not occupied.)" }</span>
                            </Button>
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
