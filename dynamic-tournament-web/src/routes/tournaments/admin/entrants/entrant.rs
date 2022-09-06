use dynamic_tournament_api::v3::tournaments::entrants::{
    Entrant as ApiEntrant, EntrantVariant, Player,
};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::components::icons::{FaPlus, FaTrash};
use crate::components::popup::Popup;
use crate::components::{Button, Input, ParseInput};

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub entrant: ApiEntrant,
    pub oncancel: Callback<()>,
    pub onsubmit: Callback<ApiEntrant>,
}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct UpdateEntrant {
    entrant: ApiEntrant,
}

impl Component for UpdateEntrant {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            entrant: ctx.props().entrant.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateName(name) => {
                match &mut self.entrant.inner {
                    EntrantVariant::Player(player) => player.name = name,
                    EntrantVariant::Team(team) => team.name = name,
                }

                true
            }
            Message::CreatePlayer => {
                match &mut self.entrant.inner {
                    EntrantVariant::Player(_) => unreachable!(),
                    EntrantVariant::Team(team) => team.players.push(Player {
                        name: String::new(),
                        role: 0.into(),
                        rating: None,
                    }),
                }

                true
            }
            Message::UpdatePlayer(index, field) => {
                let player = match &mut self.entrant.inner {
                    EntrantVariant::Player(_) => unreachable!(),
                    EntrantVariant::Team(team) => &mut team.players[index],
                };

                match field {
                    Field::Name(name) => player.name = name,
                    Field::Rating(rating) => player.rating = rating,
                }

                true
            }
            Message::DeletePlayer(index) => {
                match &mut self.entrant.inner {
                    EntrantVariant::Player(_) => unreachable!(),
                    EntrantVariant::Team(team) => team.players.remove(index),
                };

                true
            }
            Message::Cancel => {
                ctx.props().oncancel.emit(());
                false
            }
            Message::Submit => {
                ctx.props().onsubmit.emit(self.entrant.clone());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_close = ctx.link().callback(|_| Message::Cancel);

        let update_name = ctx.link().callback(Message::UpdateName);
        let body = match &self.entrant.inner {
            EntrantVariant::Player(player) => {
                html! {
                    <div>
                        <Input kind="text" value={player.name.clone()} onchange={update_name} />
                    </div>
                }
            }
            EntrantVariant::Team(team) => {
                let players: Html = team
                    .players
                    .iter().enumerate()
                    .map(|(index,player)| {
                        let update_name = ctx.link().callback(move|name|Message::UpdatePlayer(index, Field::Name(name)));
                        let update_rating = ctx.link().callback(move |rating| Message::UpdatePlayer(index, Field::Rating(Some(rating))));
                        let delete = ctx.link().callback(move |_|Message::DeletePlayer(index));

                        html! {
                            <tr>
                                <td>
                                    <Input kind="text" value={player.name.clone()} onchange={update_name} />
                                </td>
                                <td>
                                    { "WIP" }
                                </td>
                                <td>
                                    <ParseInput<u64> kind="number" value={player.rating.unwrap_or(0).to_string()} onchange={update_rating} />
                                </td>
                                <td>
                                    <Button title="Delete" onclick={delete}>
                                        <FaTrash label="Delete" />
                                    </Button>
                                </td>
                            </tr>
                        }
                    })
                    .collect();

                let create = ctx.link().callback(|_| Message::CreatePlayer);

                html! {
                    <div>
                        <h3>{ "Name" }</h3>
                        <Input kind="text" value={team.name.clone()} onchange={update_name} />

                        <h3>{ "Members" }</h3>
                        <div>
                            <Button title="Add" onclick={create}>
                                <FaPlus label="Add" />
                            </Button>
                        </div>
                        <table class="table-striped">
                            <tr>
                                <th>{ "Name" }</th>
                                <th>{ "Role" }</th>
                                <th>{ "Rating" }</th>
                                <th>{ "Delete" }</th>
                            </tr>
                            { players }
                        </table>
                    </div>
                }
            }
        };

        let onclick = ctx.link().callback(|_| Message::Submit);
        html! {
            <Popup {on_close}>
                { body }

                <button class="button" {onclick}>{ "Submit" }</button>
            </Popup>
        }
    }
}

pub enum Message {
    UpdateName(String),
    /// Create a new player in a team.
    CreatePlayer,
    UpdatePlayer(usize, Field),
    DeletePlayer(usize),
    Cancel,
    Submit,
}

pub enum Field {
    Name(String),
    Rating(Option<u64>),
}
