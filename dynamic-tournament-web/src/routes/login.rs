use crate::components::providers::{ClientProvider, Provider};
use crate::routes::Route;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::Callback;
use yew_router::components::Redirect;

pub struct Login {
    username: String,
    password: String,
    error: Option<String>,
}

impl Component for Login {
    type Message = Message;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            error: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateUsername(username) => {
                self.username = username;

                true
            }
            Message::UpdatePassword(password) => {
                self.password = password;

                true
            }
            Message::Submit => {
                let client = ClientProvider::get(ctx);

                let username = self.username.clone();
                let password = self.password.clone();

                ctx.link().send_future(async move {
                    match client.login(&username, &password).await {
                        Ok(()) => Message::ReqeustResolve,
                        Err(err) => Message::RequestReject(err.to_string()),
                    }
                });

                false
            }
            Message::ReqeustResolve => {
                // Redirect to / now.
                true
            }
            Message::RequestReject(error) => {
                self.error = Some(error);

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let client = ClientProvider::get(ctx);

        // Redirect to /.
        if client.is_authenticated() {
            return html! {
                <Redirect<Route> to={Route::Index} />
            };
        }

        let link = ctx.link().clone();

        let on_username_input = {
            let link = link.clone();
            Callback::from(move |event: InputEvent| {
                let input: HtmlInputElement = event.target_unchecked_into();
                let username = input.value();

                link.send_message(Message::UpdateUsername(username));
            })
        };

        let on_password_input = {
            let link = link.clone();
            Callback::from(move |event: InputEvent| {
                let input: HtmlInputElement = event.target_unchecked_into();
                let password = input.value();

                link.send_message(Message::UpdatePassword(password));
            })
        };

        let onsubmit = link.callback(|event: FocusEvent| {
            event.prevent_default();
            Message::Submit
        });

        let username = self.username.clone();
        let password = self.password.clone();

        let error = match self.error.clone() {
            Some(error) => html! { <span>{error}</span> },
            None => html! {},
        };

        html! {
            <div>
                <form onsubmit={onsubmit}>
                    <input
                        type="text"
                        placeholder="Username"
                        value={username}
                        oninput={on_username_input}
                    />
                    <input
                        type="password"
                        placeholder="Password"
                        value={password}
                        oninput={on_password_input}
                    />
                    <button type="submit" disabled=false>{ "Log in" }</button>
                    {error}
                </form>
            </div>
        }
    }
}

pub enum Message {
    UpdateUsername(String),
    UpdatePassword(String),
    Submit,
    ReqeustResolve,
    RequestReject(String),
}
