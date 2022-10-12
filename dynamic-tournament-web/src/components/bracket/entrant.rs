use dynamic_tournament_core::{EntrantScore, EntrantSpot};
use yew::html::{Component, Context, Html};
use yew::{html, Properties};

use std::fmt::Display;
use std::marker::PhantomData;

pub struct BracketEntrant<T>
where
    T: Display + 'static,
{
    _marker: PhantomData<T>,
}

impl<T> Component for BracketEntrant<T>
where
    T: Display + 'static,
{
    type Message = ();
    type Properties = Props<T>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let text = match &ctx.props().entrant {
            EntrantSpot::Entrant(entrant) => html! { entrant.to_string() },
            EntrantSpot::Empty => html! { "BYE" },
            EntrantSpot::TBD => html! { "TBD" },
        };

        let (score, winner) = match &ctx.props().node {
            EntrantSpot::Entrant(node) => (node.score, node.winner),
            _ => (0, false),
        };

        let classes = if winner {
            "dt-bracket-team dt-bracket-winner"
        } else {
            "dt-bracket-team"
        };

        let style = match ctx.props().color {
            Some(color) => format!("background-color: {};", color),
            None => String::from("display: hidden;"),
        };

        let x = ctx.props().x;
        let y = ctx.props().y;

        html! {
            <g>
                <text dominant-baseline="hanging" x={x.to_string()} y={y.to_string()} fill="white">{ text }</text>
                <text dominant-baseline="hanging" x={(x + 200).to_string()} y={y.to_string()} fill="white">{ score }</text>
            </g>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props<T> {
    pub entrant: EntrantSpot<T>,
    pub node: EntrantSpot<EntrantScore<u64>>,
    pub color: Option<&'static str>,
    pub x: i32,
    pub y: i32,
}

impl<T> PartialEq for Props<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
