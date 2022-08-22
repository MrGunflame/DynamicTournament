use yew::prelude::*;

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::router::Redirect;

#[derive(Debug)]
pub struct Logout;

impl Component for Logout {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        ClientProvider::get(ctx).logout();

        html! {
            <Redirect to={"/"} />
        }
    }
}
