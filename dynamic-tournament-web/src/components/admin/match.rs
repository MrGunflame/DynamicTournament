use std::{fmt::Display, marker::PhantomData};

use dynamic_tournament_generator::{render::Position, EntrantSpot};
use yew::{html, Component, Context, Html, Properties};

use super::entrant::BracketEntrant;

#[derive(Clone, Debug, Properties)]
pub struct Props<T> {
    pub entrants: [EntrantSpot<T>; 2],
    pub position: Option<Position>,
}

impl<T> PartialEq for Props<T> {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

pub struct BracketMatch<T>
where
    T: Clone + Display + 'static,
{
    _marker: PhantomData<T>,
}

impl<T> Component for BracketMatch<T>
where
    T: Clone + Display + 'static,
{
    type Message = Message;
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let entrants: Html = ctx
            .props()
            .entrants
            .iter()
            .enumerate()
            .map(|(index, entrant)| {
                html! {
                    <BracketEntrant<T> entrant={entrant.clone()} />
                }
            })
            .collect();

        let style = match ctx.props().position.unwrap_or_default() {
            Position::SpaceAround => String::from(""),
            Position::Bottom(value) => format!("position:absolite;bottom:{}%;", value),
        };

        html! {
            <div class="match" {style}>
                <div>
                    <div class="match-teams">
                        { entrants }
                    </div>
                </div>
            </div>
        }
    }
}

pub enum Message {}
