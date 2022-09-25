use yew::{html, Component, Context, Html};

use crate::api::{Action, State};
use crate::components::providers::{ClientProvider, Provider};
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
                <Link to={"/logout"}>{ "Logout" }</Link>
            }
        } else {
            html! {
                <Link to={"/login"}>{ "Login" }</Link>
            }
        };

        html! {
            <div class="navbar">
                <ul>
                    <li><Link to={"/"}>{ "Home" }</Link></li>
                    <li><Link to={"/systems"}>{ "Systems" }</Link></li>
                    <li>{ login }</li>
                </ul>
            </div>
        }
    }
}
