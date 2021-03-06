use dynamic_tournament_api::{
    v3::{
        id::EntrantId,
        tournaments::{
            entrants::{Entrant, EntrantVariant},
            EntrantKind, Tournament,
        },
    },
    Error,
};
use yew::{html, Component, Context, Html, Properties};

use crate::services::MessageLog;
use crate::utils::FetchData;
use crate::{
    components::{
        providers::{ClientProvider, Provider},
        Button,
    },
    utils::Rc,
};

#[derive(Clone, Debug, PartialEq, Properties)]
pub(super) struct Props {
    pub tournament: Rc<Tournament>,
}

#[derive(Debug)]
pub(super) struct Entrants {
    entrants: FetchData<Vec<Entrant>>,
    // Expanded teams
    expanded: Vec<bool>,
}

impl Component for Entrants {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().tournament.id;

        let client = ClientProvider::get(ctx);
        ctx.link().send_future(async move {
            match client.v3().tournaments().entrants(id).list().await {
                Ok(entrants) => Message::UpdateEntrants(FetchData::from(entrants)),
                Err(err) => Message::UpdateEntrants(FetchData::from_err(err)),
            }
        });

        Self {
            entrants: FetchData::new(),
            expanded: Vec::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateEntrants(entrants) => {
                if entrants.has_value() {
                    self.expanded = vec![false; entrants.as_ref().unwrap().len()];
                }

                self.entrants = entrants;
                true
            }
            Message::DeleteEntrant(id) => {
                let client = ClientProvider::get(ctx);

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
            Message::ExpandTeam(index) => {
                self.expanded[index] = !self.expanded[index];
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.entrants.render(|entrants| {
            let body = entrants
                .iter()
                .zip(self.expanded.iter())
                .enumerate()
                .map(|(index, (e, expanded))| {
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
                        EntrantVariant::Team(team) => {
                            let expand = ctx.link().callback(move |_| Message::ExpandTeam(index));

                            let expand = if *expanded {
                                html! {
                                    <Button title="Shrink" onclick={expand}>
                                        <i aria-hidden="true" class="fa-solid fa-caret-down" style="transition: .5s;"></i>
                                        <span class="sr-only">{ "Shrink" }</span>
                                    </Button>
                                }
                            } else {
                                html! {
                                <Button title="Expand" onclick={expand}>
                                    <i aria-hidden="true" class="fa-solid fa-caret-down" style="transform: rotate(-90deg); transition: .3s;"></i>
                                    <span class="sr-only">{ "Expand" }</span>
                                </Button>}
                            };

                            // Show all players when the team is expanded.
                            let players = if *expanded {
                                team.players
                                    .iter()
                                    .map(|player| {
                                        html! {
                                            <tr>
                                                <td>{ player.name.clone() }</td>
                                                <td>{ player.rating.unwrap_or(0) }</td>
                                            </tr>
                                        }
                                    })
                                    .collect::<Html>()
                            } else {
                                html! {}
                            };

                            html! {
                                <>
                                    <tr>
                                        <td>
                                            { expand }
                                            { team.name.clone() }
                                        </td>
                                        <td>{ team.players.len() }</td>
                                        <td>
                                            <Button title="Delete" onclick={delete}>
                                                <i aria-hidden="true" class="fa-solid fa-trash"></i>
                                                <span class="sr-only">{ "Delete" }</span>
                                            </Button>
                                        </td>
                                    </tr>
                                    { players }
                                </>
                            }
                        }
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
    ExpandTeam(usize),
}
