use yew::prelude::*;
use yew_router::components::Redirect;

use super::Route;

use dynamic_tournament_api::Client;

pub struct Logout;

impl Component for Logout {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("No ClientProvider given");

        client.logout();

        html! {
            <Redirect<Route> to={Route::Index} />
        }
    }
}
