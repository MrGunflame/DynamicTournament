use std::rc::Rc;

use dynamic_tournament_api::{
    v3::{
        id::EntrantId,
        tournaments::{
            entrants::{Entrant, EntrantVariant},
            EntrantKind, Tournament,
        },
    },
    Client, Error,
};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::components::Button;
use crate::services::MessageLog;
use crate::utils::FetchData;

#[derive(Clone, Debug, Properties)]
pub(super) struct Props {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

#[derive(Debug)]
pub(super) struct Entrants {
    entrants: FetchData<Vec<Entrant>>,
}

impl Component for Entrants {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().tournament.id;

        let (client, _) = ctx.link().context::<Client>(Callback::noop()).unwrap();
        ctx.link().send_future(async move {
            match client.v3().tournaments().entrants(id).list().await {
                Ok(entrants) => Message::UpdateEntrants(FetchData::from(entrants)),
                Err(err) => Message::UpdateEntrants(FetchData::from_err(err)),
            }
        });

        Self {
            entrants: FetchData::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateEntrants(entrants) => {
                self.entrants = entrants;
                true
            }
            Message::DeleteEntrant(id) => {
                let (client, _) = ctx.link().context::<Client>(Callback::noop()).unwrap();

                let tournament_id = ctx.props().tournament.id;
                ctx.link().send_future(async move {
                    let res = client
                        .v3()
                        .tournaments()
                        .entrants(tournament_id)
                        .delete(id)
                        .await;

                    Message::DeleteEntrantResult(res.map(|_| id))
                });

                false
            }
            Message::DeleteEntrantResult(res) => match res {
                Ok(id) => {
                    // Remove the entrant locally.
                    self.entrants
                        .as_mut()
                        .unwrap()
                        .retain(|entrant| entrant.id != id);

                    true
                }
                Err(err) => {
                    MessageLog::error(err);
                    false
                }
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.entrants.render(|entrants| {
            let body = entrants
                .iter()
                .map(|e| {
                    let id = e.id;
                    let delete = ctx.link().callback(move |_| Message::DeleteEntrant(id));

                    match &e.inner {
                        EntrantVariant::Player(player) => html! {
                            <tr>
                                <td>{ player.name.clone() }</td>
                                <td>
                                    <Button title="Delete" onclick={delete}>
                                        <i aria-hidden="true" class="fa-solid fa-trash"></i>
                                        <span class="sr-only">{ "Delete" }</span>
                                    </Button>
                                </td>
                            </tr>
                        },
                        EntrantVariant::Team(team) => html! {
                            <tr>
                                <td>{ team.name.clone() }</td>
                                <td>{ team.players.len() }</td>
                                <td>
                                    <Button title="Delete" onclick={delete}>
                                        <i aria-hidden="true" class="fa-solid fa-trash"></i>
                                        <span class="sr-only">{ "Delete" }</span>
                                    </Button>
                                </td>
                            </tr>
                        },
                    }
                })
                .collect::<Html>();

            let head = match ctx.props().tournament.kind {
                EntrantKind::Player => html! {
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Delete" }</th>
                    </tr>
                },
                EntrantKind::Team => html! {
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Players" }</th>
                        <th>{ "Delete" }</th>
                    </tr>
                },
            };

            html! {
                <div>
                    <h2>{ "Entrants" }</h2>
                    <table>
                        { head }
                        { body }
                    </table>
                </div>
            }
        })
    }
}

pub enum Message {
    UpdateEntrants(FetchData<Vec<Entrant>>),
    DeleteEntrant(EntrantId),
    DeleteEntrantResult(Result<EntrantId, Error>),
}
