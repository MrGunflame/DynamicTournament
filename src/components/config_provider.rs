use yew::prelude::*;

use crate::{render_data, Data, DataResult};

use reqwasm::http::Request;
use serde::Deserialize;

pub struct ConfigProvider {
    config: Data<Config>,
}

impl Component for ConfigProvider {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();

        link.send_future(async move {
            async fn fetch_data() -> DataResult<Config> {
                let data = Request::get("/static/config.json")
                    .send()
                    .await?
                    .json()
                    .await?;

                Ok(data)
            }

            let data = Some(fetch_data().await);

            Msg::UpdateConfig(data)
        });

        Self { config: None }
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
        render_data(&self.config, |config| {
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
    UpdateConfig(Data<Config>),
}

// #[derive(Clone, Debug)]
// pub struct Config {
//     inner: Rc<InnerConfig>,
// }

// #[derive(Clone, Debug, Deserialize)]
// pub struct InnerConfig {
//     api_url: String,
// }

// impl PartialEq for Config {
//     fn eq(&self, other: &Self) -> bool {
//         Rc::ptr_eq(&self.inner, &other.inner)
//     }
// }

// impl std::ops::Deref for Config {
//     type Target = InnerConfig;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }
