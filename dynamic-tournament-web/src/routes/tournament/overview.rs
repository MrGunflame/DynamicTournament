use std::rc::Rc;

use dynamic_tournament_api::tournament::Tournament;
use yew::prelude::*;

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
                { ctx.props().tournament.id }
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
