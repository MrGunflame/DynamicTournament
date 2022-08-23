use dynamic_tournament_api::v3::id::{EntrantId, TournamentId};
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use yew::{html, Component, Context, Html, Properties};

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::router::RouterContextExt;
use crate::utils::FetchData;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament_id: TournamentId,
}

#[derive(Debug)]
pub struct Entrants {
    entrants: FetchData<Vec<Entrant>>,
}

impl Component for Entrants {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let id = ctx.props().tournament_id;
        ctx.link().send_future(async move {
            let msg = match client.v3().tournaments().entrants(id).list().await {
                Ok(val) => FetchData::from(val),
                Err(err) => FetchData::from_err(err),
            };

            Message::UpdateEntrants(msg)
        });

        Self {
            entrants: FetchData::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Message) -> bool {
        match msg {
            Message::UpdateEntrants(entrants) => {
                self.entrants = entrants;
                true
            }
            Message::OnClick(id) => {
                ctx.router().update(|path| {
                    path.push(id.to_string());
                });

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.entrants.render(|entrants| {
            let entrants: Html = entrants
                .iter()
                .map(|entrant| {
                    let id = entrant.id;
                    let onclick = ctx.link().callback(move |_| Message::OnClick(id));

                    match &entrant.inner {
                        EntrantVariant::Player(player) => html! {
                            <tr {onclick} class="tr-link">
                                <td>{player.name.clone()}</td>
                                <td>{1}</td>
                            </tr>
                        },
                        EntrantVariant::Team(team) => html! {
                            <tr {onclick} class="tr-link">
                                <td>{ team.name.clone() }</td>
                                <td>{ team.players.len() }</td>
                            </tr>
                        },
                    }
                })
                .collect();

            html! {
                <table class="table-center">
                    <thead>
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "Players" }</th>
                        </tr>
                    </thead>
                    <tbody>
                        { entrants }
                    </tbody>
                </table>
            }
        })
    }
}

pub enum Message {
    UpdateEntrants(FetchData<Vec<Entrant>>),
    OnClick(EntrantId),
}
