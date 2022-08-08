use yew::{html, Component, Context, Html};
use yew_router::components::Link;

use crate::components::providers::{ClientProvider, Provider};
use crate::routes::Route;

#[derive(Debug)]
pub struct Navbar;

impl Component for Navbar {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
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
                    <li><Link<Route> to={Route::TournamentList}>{ "Tournaments" }</Link<Route>></li>
                    <li><Link<Route> to={Route::Systems}>{ "Systems" }</Link<Route>></li>
                    <li>{ login }</li>
                </ul>
            </div>
        }
    }
}
