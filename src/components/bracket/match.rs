use gloo_storage::Storage;
use yew::prelude::*;

use crate::api::tournament::Team;
use crate::api::v1::auth::AuthCredentials;
use crate::bracket_generator::{EntrantSpot, EntrantWithScore};

use super::double_elimination::Action;
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

        let credentails: gloo_storage::Result<AuthCredentials> =
            gloo_storage::LocalStorage::get("http_auth_data");

        let action_button = match credentails {
            Ok(_) => {
                if ctx.props().entrants[0].is_entrant() && ctx.props().entrants[1].is_entrant() {
                    let onclick = ctx.link().callback(|_| Message::UpdateScore);

                    html! { <button onclick={onclick} disabled=false>{"Update Score"}</button> }
                } else {
                    html! { <button title="Some entrant spots are not occupied." disabled=true>{"Update Score"}</button> }
                }
            }
            Err(err) => {
                gloo_console::warn!(format!("Failed to read authorization credentials: {}", err));

                html! {
                    <button title="You are not logged in (or an error occured)." disabled=true>{"Update Score"}</button>
                }
            }
        };

        html! {
            <div class="match">
                {entrants}
                {action_button}
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub entrants: [EntrantSpot<EntrantWithScore<Team, u64>>; 2],
    pub on_action: Callback<Action>,
}

pub enum Message {
    UpdateScore,
}
