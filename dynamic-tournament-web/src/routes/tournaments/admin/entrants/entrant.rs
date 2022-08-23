use dynamic_tournament_api::v3::tournaments::entrants::{Entrant as ApiEntrant, EntrantVariant};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::components::popup::Popup;
use crate::components::Input;

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
            Message::UpdatePlayerName(index, name) => {
                match &mut self.entrant.inner {
                    EntrantVariant::Player(_) => unreachable!(),
                    EntrantVariant::Team(team) => team.players[index].name = name,
                }

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
                        let update_name = ctx.link().callback(move|name|Message::UpdatePlayerName(index,name));

                        html! {
                            <tr>
                                <td>
                                    <Input kind="text" value={player.name.clone()} onchange={update_name} />
                                </td>
                            </tr>
                        }
                    })
                    .collect();

                html! {
                    <div>
                        <Input kind="text" value={team.name.clone()} onchange={update_name} />

                        <table class="table-striped">
                            <tr>
                                <th>{ "Name" }</th>
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
    UpdatePlayerName(usize, String),
    Cancel,
    Submit,
}
