use std::rc::Rc;

use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::utils::FetchData;

use super::Route;

use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use dynamic_tournament_api::Client;

pub struct Entrants {
    entrants: FetchData<Vec<Entrant>>,
}

impl Component for Entrants {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let (client, _) = link
            .context::<Client>(Callback::noop())
            .expect("no client in context");

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

                ctx.link()
                    .history()
                    .expect("No History given")
                    .push(Route::TeamDetails {
                        tournament_id: id,
                        tournament_name: name,
                        team_id,
                    });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.entrants.render(|entrants| {
            let entrants: Html = entrants
                .iter()
                .enumerate()
                .map(|(index, entrant)| {
                    let onclick = ctx.link().callback(move |_| Message::OnClick(index));

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

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

pub enum Message {
    Update(FetchData<Vec<Entrant>>),
    OnClick(usize),
}
