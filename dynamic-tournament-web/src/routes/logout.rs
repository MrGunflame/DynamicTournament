use yew::prelude::*;
use yew_router::components::Redirect;

use crate::components::providers::{ClientProvider, Provider};

use super::Route;

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
            <Redirect<Route> to={Route::Index} />
        }
    }
}
