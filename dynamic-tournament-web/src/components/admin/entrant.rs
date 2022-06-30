use std::{fmt::Display, marker::PhantomData};

use dynamic_tournament_generator::EntrantSpot;
use yew::{html, Component, Context, Html, Properties};

#[derive(Clone, Debug, Properties)]
pub struct Props<T> {
    pub entrant: EntrantSpot<T>,
}

impl<T> PartialEq for Props<T> {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

pub struct BracketEntrant<T>
where
    T: Clone + Display + 'static,
{
    _marker: PhantomData<T>,
}

impl<T> Component for BracketEntrant<T>
where
    T: Clone + Display + 'static,
{
    type Message = ();
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let text = match &ctx.props().entrant {
            EntrantSpot::Entrant(entrant) => entrant.to_string(),
            EntrantSpot::Empty => String::from("BYE"),
            EntrantSpot::TBD => String::from("TBD"),
        };

        html! {
            <div class="team">
                <div class="team-label flex-col">
                    <span>{ text }</span>
                </div>
            </div>
        }
    }
}
