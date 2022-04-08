use crate::api::v1::auth::LoginData;
use crate::components::config_provider::Config;
use crate::components::providers::auth::{Auth, InnerAuth};
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
                let (config, _) = ctx
                    .link()
                    .context::<Config>(Callback::noop())
                    .expect("No ConfigProvider given");

                let username = self.username.clone();
                let password = self.password.clone();

                let logindata = LoginData::new(username.clone(), password.clone());

                ctx.link().send_future(async move {
                    async fn fetch_data(
                        logindata: LoginData,
                        config: Config,
                    ) -> Result<(), Box<dyn std::error::Error>> {
                        logindata.post(config).await?;

                        Ok(())
                    }

                    match fetch_data(logindata, config).await {
                        Ok(_) => Message::ReqeustResolve(InnerAuth { username, password }),
                        Err(err) => Message::RequestReject(err.to_string()),
                    }
                });

                false
            }
            Message::ReqeustResolve(data) => {
                let (auth, _) = ctx
                    .link()
                    .context::<Auth>(Callback::noop())
                    .expect("No AuthContext provided");

                let mut inner = auth.inner.lock().unwrap();
                *inner = Some(data);

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
        // Move to different block here so we drop the mutexguard after the check.
        {
            let (auth, _) = ctx
                .link()
                .context::<Auth>(Callback::noop())
                .expect("No AuthContext provided");

            // Redirect to /.
            if auth.inner.lock().unwrap().is_some() {
                return html! {
                    <Redirect<Route> to={Route::Index} />
                };
            }
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
    RequestReject(String),
    ReqeustResolve(InnerAuth),
}
