use crate::components::config_provider::Config;
use crate::routes::tournament::Route;
use crate::{render_data, Data, DataResult};
use reqwasm::http::Request;
use yew::prelude::*;
use yew_router::components::Link;

pub struct TournamentList {
    data: Data<Vec<u64>>,
}

impl Component for TournamentList {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let (config, _) = ctx
            .link()
            .context(Callback::noop())
            .expect("No ConfigProvider given");

        link.send_future(async move {
            async fn fetch_data(config: Config) -> DataResult<Vec<u64>> {
                let data = Request::get(&format!("{}/api/v1/tournament", config.api_url))
                    .send()
                    .await?
                    .json()
                    .await?;

                Ok(data)
            }

            let data = Some(fetch_data(config).await);

            Msg::Update(data)
        });

        Self { data: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(data) => {
                self.data = data;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        render_data(&self.data, |data| {
            let tournaments: Html = data
                .iter()
                .map(|id| {
                    html! {
                        <Link<Route> classes="link-inline" to={Route::Index { id: *id } }>{ id }</Link<Route>>
                    }
                })
                .collect();

            html! {
                <div>
                        {tournaments}
                </div>
            }
        })
    }
}

pub enum Msg {
    Update(Data<Vec<u64>>),
}
