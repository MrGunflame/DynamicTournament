mod entrant;

use dynamic_tournament_api::{
    v3::{
        id::EntrantId,
        tournaments::{
            entrants::{Entrant, EntrantVariant, Player, Team},
            EntrantKind, Tournament,
        },
    },
    Error,
};
use yew::{html, Component, Context, Html, Properties};

use self::entrant::UpdateEntrant;
use crate::components::icons::{FaPenToSquare, FaPlus, FaTrash};
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
    // index of the entrant being updated
    update_entrant: Option<PopupState>,
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
            update_entrant: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateEntrants(entrants) => {
                self.entrants = entrants;
                true
            }
            Message::CreateEntrant(entrant) => {
                let client = ClientProvider::get(ctx);

                let tournament_id = ctx.props().tournament.id;
                ctx.link().send_future(async move {
                    let res = client
                        .v3()
                        .tournaments()
                        .entrants(tournament_id)
                        .create(&entrant)
                        .await;

                    Message::CreateEntrantResult(res)
                });

                false
            }
            Message::CreateEntrantResult(res) => match res {
                Ok(entrant) => {
                    self.entrants.as_mut().unwrap().push(entrant);
                    self.update_entrant = None;

                    true
                }
                Err(err) => {
                    MessageLog::error(err);
                    false
                }
            },
            Message::PatchEntrant(entrant) => {
                let client = ClientProvider::get(ctx);

                let tournament_id = ctx.props().tournament.id;
                ctx.link().send_future(async move {
                    let res = client
                        .v3()
                        .tournaments()
                        .entrants(tournament_id)
                        .patch(entrant.id, &entrant)
                        .await;

                    Message::PatchEntrantResult(res)
                });

                false
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
            Message::PatchEntrantResult(res) => match res {
                Ok(entrant) => {
                    for e in self.entrants.as_mut().unwrap() {
                        if e.id == entrant.id {
                            *e = entrant;
                            break;
                        }
                    }

                    self.update_entrant = None;

                    true
                }
                Err(err) => {
                    MessageLog::error(err);
                    false
                }
            },
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
            Message::CreateEntrantStart => {
                self.update_entrant = Some(PopupState::Create);
                true
            }
            Message::UpdateEntrantStart(index) => {
                self.update_entrant = Some(PopupState::Update(index));
                true
            }
            Message::UpdateEntrantCancel => {
                self.update_entrant = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.entrants.render(|entrants| {
            let body = entrants
                .iter()
                .enumerate()
                .map(|(index, e)| {
                    let id = e.id;
                    let delete = ctx.link().callback(move |_| Message::DeleteEntrant(id));
                    let edit = ctx.link().callback(move |_|Message::UpdateEntrantStart(index));

                    match &e.inner {
                        EntrantVariant::Player(player) => html! {
                            <tr>
                                <td>{ player.name.clone() }</td>
                                <td>{ player.rating.unwrap_or(0) }</td>
                                <td>
                                    <Button title="Edit" onclick={edit}>
                                        <FaPenToSquare label="Edit" />
                                    </Button>
                                </td>
                                <td>
                                    <Button title="Delete" onclick={delete}>
                                        <FaTrash label="Delete" />
                                    </Button>
                                </td>
                            </tr>
                        },
                        EntrantVariant::Team(team) => {

                            html! {
                                <>
                                    <tr>
                                        <td>
                                            { team.name.clone() }
                                        </td>
                                        <td>{ e.rating().unwrap_or(0) }</td>
                                        <td>{ team.players.len() }</td>
                                        <td>
                                            <Button title="Edit" onclick={edit}>
                                                <FaPenToSquare label="Edit" />
                                            </Button>
                                        </td>
                                        <td>
                                            <Button title="Delete" onclick={delete}>
                                                <FaTrash label="Delete" />
                                            </Button>
                                        </td>
                                    </tr>
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
                        <th>{ "Rating" }</th>
                        <th>{ "Edit" }</th>
                        <th>{ "Delete" }</th>
                    </tr>
                },
                EntrantKind::Team => html! {
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Rating" }</th>
                        <th>{ "Players" }</th>
                        <th>{ "Edit" }</th>
                        <th>{ "Delete" }</th>
                    </tr>
                },
            };

            let popup = match self.update_entrant {
                Some(state) =>  {
                    let oncancel = ctx.link().callback(|_|Message::UpdateEntrantCancel);

                    match state {
                        PopupState::Create => {
                            let entrant = match ctx.props().tournament.kind {
                                EntrantKind::Player => Entrant::player(Player { name: String::new(), role: 0.into(), rating: None }),
                                EntrantKind::Team => Entrant::team(Team { name: String::new(), players: Vec::new() }),
                            };

                            let onsubmit = ctx.link().callback(Message::CreateEntrant);

                            html! {
                                <UpdateEntrant {entrant} {oncancel} {onsubmit} />
                            }
                        }
                        PopupState::Update(index) => {
                            let entrant = self.entrants.as_ref().unwrap()[index].clone();
                            let onsubmit = ctx.link().callback(Message::PatchEntrant);

                            html! {
                                <UpdateEntrant {entrant} {oncancel} {onsubmit} />
                            }
                        }
                    }

                }
                None => html! {},
            };

            let create = ctx.link().callback(|_|Message::CreateEntrantStart);

            html! {
                <>
                    { popup }

                    <div>
                        <h2>{ "Entrants" }</h2>
                        <p>
                            { "Deleted entrants that are still placed in a bracket will be replaced with a placeholder string. "}
                            <strong>{ "Once deleted they cannot be stored." }</strong>
                        </p>
                        <div class="admin-entrants-actions">
                            <Button title="Add" onclick={create}>
                                <FaPlus label="Add" />
                            </Button>
                        </div>
                        <table class="table-striped">
                            { head }
                            { body }
                        </table>
                    </div>
                </>
            }
        })
    }
}

pub enum Message {
    UpdateEntrants(FetchData<Vec<Entrant>>),
    CreateEntrant(Entrant),
    CreateEntrantResult(Result<Entrant, Error>),
    PatchEntrant(Entrant),
    PatchEntrantResult(Result<Entrant, Error>),
    DeleteEntrant(EntrantId),
    DeleteEntrantResult(Result<EntrantId, Error>),
    UpdateEntrantStart(usize),
    CreateEntrantStart,
    UpdateEntrantCancel,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum PopupState {
    Create,
    Update(usize),
}
