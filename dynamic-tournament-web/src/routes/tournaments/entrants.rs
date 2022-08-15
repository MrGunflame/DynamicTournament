use dynamic_tournament_api::v3::id::EntrantId;
use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::prelude::*;

use crate::utils::router::RouterContextExt;
use crate::utils::Rc;
use crate::{
    components::providers::{ClientProvider, Provider},
    utils::FetchData,
};

use super::Route;

use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};

pub struct Entrants {
    entrants: FetchData<Vec<Entrant>>,
}

impl Component for Entrants {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let client = ClientProvider::get(ctx);

        let tournament_id = ctx.props().tournament.id;
        link.send_future(async move {
            let entrants = match client
                .v3()
                .tournaments()
                .entrants(tournament_id)
                .list()
                .await
            {
                Ok(entrants) => FetchData::from(entrants),
                Err(err) => FetchData::from_err(err),
            };

            Message::Update(entrants)
        });

        Self {
            entrants: FetchData::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Update(entrants) => {
                self.entrants = entrants;
            }
            Message::OnClick(team_id) => {
                let id = ctx.props().tournament.id;
                let name = ctx.props().tournament.name.clone();

                ctx.history()
                    .redirect(Route::TeamDetails { id, name, team_id });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.entrants.render(|entrants| {
            let entrants: Html = entrants
                .iter()
                .map(|entrant| {
                    let id = entrant.id;

                    let onclick = ctx.link().callback(move |_| Message::OnClick(id));

                    match entrant.inner {
                        EntrantVariant::Player(ref player) => html! {
                            <tr {onclick} class="tr-link">
                                <td>{ player.name.clone() }</td>
                                <td>{1}</td>
                            </tr>
                        },
                        EntrantVariant::Team(ref team) => html! {
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
                    <tbody>
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "Players" }</th>
                        </tr>
                        { entrants }
                    </tbody>
                </table>
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

pub enum Message {
    Update(FetchData<Vec<Entrant>>),
    OnClick(EntrantId),
}
