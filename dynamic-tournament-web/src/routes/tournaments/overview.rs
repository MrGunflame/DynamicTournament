use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::{html, Component, Context, Html, Properties};

use crate::utils::Rc;

pub struct Overview;

impl Component for Overview {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                { ctx.props().tournament.description.clone() }
            </>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}
