use crate::components::providers::{ClientProvider, Provider};
use crate::utils::FetchData;
use crate::Title;
use chrono::Local;
use yew::prelude::*;

use dynamic_tournament_api::v3::id::TournamentId;
use dynamic_tournament_api::v3::tournaments::TournamentOverview;

use crate::utils::router::RouterContextExt;

pub struct TournamentList {
    tournaments: FetchData<Vec<TournamentOverview>>,
    timezone: Local,
}

impl Component for TournamentList {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Title::set("Tournaments");

        let link = ctx.link();
        let client = ClientProvider::get(ctx);

        link.send_future(async move {
            let msg = match client.v3().tournaments().list().await {
                Ok(tournaments) => FetchData::from(tournaments),
                Err(err) => FetchData::from_err(err),
            };

            Message::Update(msg)
        });

        Self {
            tournaments: FetchData::new(),
            timezone: Local::now().timezone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Update(tournaments) => {
                self.tournaments = tournaments;
                true
            }
            Message::ClickTournament { id } => {
                ctx.history().push(format!("/tournaments/{}", id));

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.tournaments.render(|tournaments| {
            let tournaments: Html = tournaments
                .iter()
                .map(|tournament| {
                    let id = tournament.id;
                    let name = tournament.name.clone();
                    let bracket_type = tournament.kind.to_string();
                    let date = tournament
                        .date
                        .with_timezone(&self.timezone)
                        .format("%B %d, %Y %H:%M");

                    let on_click = ctx
                        .link()
                        .callback(move |_| Message::ClickTournament { id });

                    html! {
                        <tr class="tr-link" onclick={on_click}>
                            <td>{ name }</td>
                            <td>{ bracket_type }</td>
                            <td>{ date }</td>
                        </tr>
                    }
                })
                .collect();

            html! {
                <div>
                    <h1>{ "Tournaments" }</h1>
                    <table class="tr-link-table tr-table">
                        <tr class="table-head">
                            <th>{ "Name" }</th>
                            <th>{ "Type" }</th>
                            <th>{ "Date" }</th>
                        </tr>
                        {tournaments}
                    </table>
                </div>
            }
        })
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        Title::clear();
    }
}

pub enum Message {
    Update(FetchData<Vec<TournamentOverview>>),
    ClickTournament { id: TournamentId },
}
