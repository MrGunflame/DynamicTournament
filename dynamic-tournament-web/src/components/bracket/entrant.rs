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
            EntrantSpot::Empty => html! { <i>{ "BYE" }</i> },
            EntrantSpot::TBD => html! { <i>{ "TBD" }</i> },
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

        html! {
            <div class={classes}>
                <div class="dt-bracket-team-name dt-flex">
                    <div class="dt-bracket-team-color" { style }></div>
                    <span>{ text }</span>
                </div>
                <div class="dt-bracket-team-score">{ score }</div>
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props<T> {
    pub entrant: EntrantSpot<T>,
    pub node: EntrantSpot<EntrantScore<u64>>,
    pub color: Option<&'static str>,
}

impl<T> PartialEq for Props<T> {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
