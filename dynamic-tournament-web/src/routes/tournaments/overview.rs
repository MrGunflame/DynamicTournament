use std::rc::Rc;

use yew::prelude::*;

use dynamic_tournament_api::v3::tournaments::Tournament;

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

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}