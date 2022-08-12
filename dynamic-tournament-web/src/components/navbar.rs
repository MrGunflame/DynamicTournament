use yew::{html, Component, Context, Html};

use crate::api::Action;
use crate::components::providers::{ClientProvider, Provider};
use crate::routes::Route;
use crate::utils::router::Link;

#[derive(Debug)]
pub struct Navbar;

impl Component for Navbar {
    type Message = Action;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let mut client = ClientProvider::get(ctx);

        let link = ctx.link().clone();
        ctx.link().send_future_batch(async move {
            loop {
                let action = client.changed().await;
                link.send_message(action);
            }
        });

        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Action) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let client = ClientProvider::get(ctx);

        let login = if client.is_authenticated() {
            html! {
                <Link<Route> to={Route::Logout}>{ "Logout" }</Link<Route>>
            }
        } else {
            html! {
                <Link<Route> to={Route::Login}>{ "Login" }</Link<Route>>
            }
        };

        html! {
            <div class="navbar">
                <ul>
                    <li><Link<Route> to={Route::Index}>{ "Home" }</Link<Route>></li>
                    <li><Link<Route> to={Route::Tournaments}>{ "Tournaments" }</Link<Route>></li>
                    <li><Link<Route> to={Route::Systems}>{ "Systems" }</Link<Route>></li>
                    <li>{ login }</li>
                </ul>
            </div>
        }
    }
}
