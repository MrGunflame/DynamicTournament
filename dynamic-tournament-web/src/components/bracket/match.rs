use dynamic_tournament_core::render::Position;
use yew::prelude::*;

use dynamic_tournament_core::{EntrantScore, EntrantSpot};

use super::BracketEntrant;
use crate::components::button::Button;
use crate::components::providers::{ClientProvider, Provider};

use std::fmt::Display;
use std::marker::PhantomData;

const COLOR_RED: &str = "#a52423";
const COLOR_BLUE: &str = "#193d6b";

/// A single match of a tournament (also called tie, fixture or heat).
pub struct BracketMatch<T>
where
    T: Clone + Display + 'static,
{
    _maker: PhantomData<T>,
}

impl<T> Component for BracketMatch<T>
where
    T: Clone + Display + 'static,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _maker: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let action = match msg {
            Message::UpdateScore => Action::UpdateMatch,
            Message::ResetMatch => Action::ResetMatch,
        };

        ctx.props().on_action.emit(action);
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let entrants: Html = ctx
            .props()
            .entrants
            .iter()
            .zip(ctx.props().nodes)
            .enumerate()
            .map(|(index, (entrant, node))| {
                let color = match index {
                    0 => Some(COLOR_RED),
                    _ => Some(COLOR_BLUE),
                };

                html! {
                    <BracketEntrant<T> entrant={entrant.clone()} {node} {color} />
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

        let number = ctx.props().number;

        let style = match ctx.props().position.unwrap_or_default() {
            Position::SpaceAround => String::from(""),
            Position::Bottom(value) => format!("position:absolute;bottom:{}%;", value),
        };

        html! {
            <div class="match" {style}>
                <span>{ number }</span>
                <div>
                    <div class="match-teams">
                        {entrants}
                    </div>
                    {action_button}
                </div>
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props<T> {
    pub entrants: [EntrantSpot<T>; 2],
    pub nodes: [EntrantSpot<EntrantScore<u64>>; 2],
    pub on_action: Callback<Action>,
    pub number: usize,
    pub position: Option<Position>,
}

impl<T> PartialEq for Props<T> {
    fn eq(&self, _other: &Self) -> bool {
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
    ResetMatch,
}
