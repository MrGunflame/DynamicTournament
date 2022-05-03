use yew::context::ContextProvider;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};

use super::Provider;
use crate::components::config_provider::Config;
use crate::services::client::ClientEventBus;

use dynamic_tournament_api::{auth::Token, Client};
use gloo_timers::future::sleep;

pub struct ClientProvider {
    client: Client,
    _producer: Box<dyn Bridge<ClientEventBus>>,
}

impl Component for ClientProvider {
    type Message = ();
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let (config, _) = ctx
            .link()
            .context::<Config>(Callback::noop())
            .expect("No ConfigProvider given");

        let client = Client::new(config.api_url);

        if let Some(auth) = client.authorization().auth_token() {
            let now = chrono::Utc::now().timestamp() as u64;
            let token = Token::new(auth);
            let claims = token.claims();
            // 30 secs buffer time
            let duration = claims.exp.saturating_sub(now + 30);

            log::debug!("Auth token is valid for {}s", duration);

            ctx.link().send_future(async move {
                sleep(std::time::Duration::new(duration, 0)).await;
            });
        }

        Self {
            client,
            _producer: ClientEventBus::bridge(ctx.link().callback(|_| ())),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        log::debug!("Refreshing authorization tokens");

        let client = self.client.clone();
        let link = ctx.link().clone();
        ctx.link().send_future_batch(async move {
            if let Err(err) = client.auth().refresh().await {
                log::error!("Failed to refresh authorization tokens: {:?}", err);
            }

            if let Some(auth) = client.authorization().auth_token() {
                let now = chrono::Utc::now().timestamp() as u64;
                let token = Token::new(auth);
                let claims = token.claims();
                // 30 secs buffer time
                let duration = claims.exp.saturating_sub(now + 30);

                log::debug!("Auth token is valid for {}s", duration);

                link.send_future(async move {
                    sleep(std::time::Duration::new(duration, 0)).await;
                });
            }

            vec![]
        });

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<Client> context={self.client.clone()}>
                { for ctx.props().children.iter() }
            </ContextProvider<Client>>
        }
    }
}

impl<C> Provider<Client, C> for ClientProvider
where
    C: Component,
{
    fn take(ctx: &Context<C>) -> Client {
        let (client, _) = ctx
            .link()
            .context(Callback::noop())
            .expect("No ClientProvider given");

        client
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Properties {
    pub children: Children,
}
