use crate::routes::tournament::Route;
use crate::{render_data, Data, DataResult};
use yew::prelude::*;
use yew_router::components::Link;

use dynamic_tournament_api::tournament::TournamentId;
use dynamic_tournament_api::Client;

pub struct TournamentList {
    data: Data<Vec<TournamentId>>,
}

impl Component for TournamentList {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let (client, _) = ctx
            .link()
            .context(Callback::noop())
            .expect("No ClientProvider given");

        link.send_future(async move {
            async fn fetch_data(client: Client) -> DataResult<Vec<TournamentId>> {
                let client = client.tournaments();

                let data = client.list().await?;

                Ok(data)
            }

            let data = Some(fetch_data(client).await);

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
                        <Link<Route> classes="link-inline" to={Route::Index { id: id.0 } }>{ id.0 }</Link<Route>>
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
    Update(Data<Vec<TournamentId>>),
}
