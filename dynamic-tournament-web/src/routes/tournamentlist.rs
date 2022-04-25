use crate::routes::tournament::Route;
use crate::{render_data, Data, DataResult, Title};
use yew::prelude::*;

use dynamic_tournament_api::tournament::{TournamentId, TournamentOverview};
use dynamic_tournament_api::Client;
use yew_router::history::History;
use yew_router::prelude::RouterScopeExt;

pub struct TournamentList {
    data: Data<Vec<TournamentOverview>>,
}

impl Component for TournamentList {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Title::set("Tournaments");

        let link = ctx.link();
        let (client, _) = ctx
            .link()
            .context(Callback::noop())
            .expect("No ClientProvider given");

        link.send_future(async move {
            async fn fetch_data(client: Client) -> DataResult<Vec<TournamentOverview>> {
                let client = client.tournaments();

                let data = client.list().await?;

                Ok(data)
            }

            let data = Some(fetch_data(client).await);

            Msg::Update(data)
        });

        Self { data: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(data) => {
                self.data = data;
                true
            }
            Msg::ClickTournament { id } => {
                ctx.link()
                    .history()
                    .expect("failed to read history")
                    .push(Route::Index { id: id.0 });

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        render_data(&self.data, |data| {
            let tournaments: Html = data
                .iter()
                .map(|tournament| {
                    let id = tournament.id;
                    let name = tournament.name.clone();
                    let bracket_type = tournament.bracket_type.to_string();
                    let teams = tournament.teams;

                    let on_click = ctx.link().callback(move |_| Msg::ClickTournament { id });

                    html! {
                        <tr class="tr-link" onclick={on_click}>
                            <td>{ name }</td>
                            <td>{ bracket_type }</td>
                            <td>{ "WIP" }</td>
                            <td>{ teams }</td>
                        </tr>
                    }
                })
                .collect();

            html! {
                <table class="tr-link-table table-center">
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Type" }</th>
                        <th>{ "Date" }</th>
                        <th>{ "Teams" }</th>
                    </tr>
                    {tournaments}
                </table>
            }
        })
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        Title::clear();
    }
}

pub enum Msg {
    Update(Data<Vec<TournamentOverview>>),
    ClickTournament { id: TournamentId },
}
