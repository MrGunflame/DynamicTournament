use dynamic_tournament_core::render::Position;
use yew::prelude::*;

use dynamic_tournament_api::auth::Flags;
use dynamic_tournament_core::{EntrantScore, EntrantSpot};

use super::BracketEntrant;
use crate::components::button::Button;
use crate::components::icons::{FaPen, FaRotateLeft, FaSize};
use crate::components::Protected;

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
                    <BracketEntrant<T> entrant={entrant.clone()} {node} {color} x={0} y={(index * 20) as i32} />
                }
            })
            .collect();

        let action_button = if ctx.props().entrants[0].is_entrant()
            && ctx.props().entrants[1].is_entrant()
        {
            let onclick = ctx.link().callback(|_| Message::UpdateScore);

            let on_reset = ctx.link().callback(|_| Message::ResetMatch);

            html! {
                <Protected flags={Flags::EDIT_SCORES}>
                    <div class="dt-bracket-match-actions">
                        <Button classes="" {onclick} title="Edit">
                            <FaPen label="Edit" size={FaSize::ExtraLarge} />
                        </Button>
                        <Button classes="" onclick={on_reset} title="Reset">
                            <FaRotateLeft label="Reset" size={FaSize::ExtraLarge} />
                        </Button>
                    </div>
                </Protected>
            }
        } else {
            html! {
                <Protected flags={Flags::EDIT_SCORES}>
                    <div class="dt-bracket-match-actions" style="filter: contrast(0%);">
                        <Button classes="" title="Edit (Some entrant spots are not occupied.)" disabled=true>
                            <FaPen label="Edit (Some entrant spots are not occupied.)" size={FaSize::ExtraLarge} />
                        </Button>
                        <Button classes="" title="Reset (Some entrant spots are not occupied.)" disabled=true>
                            <FaRotateLeft label="Reset (Some entrant spots are not occupied.)" size={FaSize::ExtraLarge} />
                        </Button>
                    </div>
                </Protected>
            }
        };

        let number = ctx.props().number;

        let style = match ctx.props().position.unwrap_or_default() {
            Position::SpaceAround => String::from(""),
            Position::Bottom(value) => format!("position:absolute;bottom:{}%;", value),
        };

        let x = ctx.props().x.to_string();
        let y = ctx.props().y.to_string();

        let transform = format!("translate({},{})", x, y);

        html! {
            <g x={x.clone()} y={y.clone()} {transform} class="match" {style}>
                <text y="50%" fill="white">{ number }</text>
                {entrants}
            </g>
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
    pub x: usize,
    pub y: usize,
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
