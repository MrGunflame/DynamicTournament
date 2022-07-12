use yew::context::ContextProvider;
use yew::prelude::*;

use super::Provider;
use crate::api::Client;
use crate::components::config_provider::Config;

#[derive(Debug)]
pub struct ClientProvider {
    client: Client,
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

        Self { client }
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
    fn get(ctx: &Context<C>) -> Client {
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
