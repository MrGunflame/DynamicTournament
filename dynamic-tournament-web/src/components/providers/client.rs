use yew::context::ContextProvider;
use yew::prelude::*;

use crate::components::config_provider::Config;

use dynamic_tournament_api::Client;

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

        Self {
            client: Client::new(config.api_url),
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<Client> context={self.client.clone()}>
                { for ctx.props().children.iter() }
            </ContextProvider<Client>>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Properties {
    pub children: Children,
}
