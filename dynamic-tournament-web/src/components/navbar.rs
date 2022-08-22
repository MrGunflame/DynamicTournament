use yew::{html, Component, Context, Html};

use crate::api::{Action, State};
use crate::components::providers::{ClientProvider, Provider};
use crate::routes::Route;
use crate::utils::router::Link;

#[derive(Debug)]
pub struct Navbar {
    state: State,
}

impl Component for Navbar {
    type Message = Action;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);
        let state = client.state();

        let link = ctx.link().clone();
        ctx.link().send_future_batch(async move {
            loop {
                let action = client.changed().await;
                link.send_message(action);
            }
        });

        Self { state }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Action::Login => self.state = State::LoggedIn,
            Action::Logout => self.state = State::LoggedOut,
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let login = if self.state == State::LoggedIn {
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
