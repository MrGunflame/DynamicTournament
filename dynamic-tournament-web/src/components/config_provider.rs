use yew::prelude::*;

use crate::utils::FetchData;

use reqwasm::http::Request;
use serde::Deserialize;

#[derive(Debug)]
pub struct ConfigProvider {
    config: FetchData<Config>,
}

impl Component for ConfigProvider {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();

        link.send_future(async move {
            let data = match Request::get("/config.json").send().await {
                Ok(resp) => match resp.json::<Config>().await {
                    Ok(body) => FetchData::from(body),
                    Err(err) => FetchData::from_err(err),
                },
                Err(err) => FetchData::from_err(err),
            };

            Msg::UpdateConfig(data)
        });

        Self {
            config: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateConfig(config) => {
                self.config = config;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.config.render(|config| {
            html! {
                <ContextProvider<Config> context={config.clone()}>
                    { for ctx.props().children.iter() }
                </ContextProvider<Config>>
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Config {
    pub api_url: String,
}

pub enum Msg {
    UpdateConfig(FetchData<Config>),
}
