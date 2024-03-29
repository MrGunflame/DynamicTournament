use std::collections::HashMap;

use dynamic_tournament_api::v3::id::{EntrantId, RoleId, TournamentId};
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant as ApiEntrant, EntrantVariant};
use yew::{html, Component, Context, Html, Properties};

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::FetchData;

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament_id: TournamentId,
    pub entrant_id: EntrantId,
}

#[derive(Debug)]
pub struct Entrant {
    entrant: FetchData<ApiEntrant>,
    roles: FetchData<HashMap<RoleId, String>>,
}

impl Component for Entrant {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let tournament_id = ctx.props().tournament_id;
        let entrant_id = ctx.props().entrant_id;

        {
            let client = client.clone();
            ctx.link().send_future(async move {
                let msg = match client
                    .v3()
                    .tournaments()
                    .entrants(tournament_id)
                    .get(entrant_id)
                    .await
                {
                    Ok(val) => FetchData::from(val),
                    Err(err) => FetchData::from_err(err),
                };

                Message::UpdateEntrant(msg)
            });
        }

        ctx.link().send_future(async move {
            let msg = match client.v3().tournaments().roles(tournament_id).list().await {
                Ok(val) => FetchData::from(
                    val.into_iter()
                        .map(|role| (role.id, role.name))
                        .collect::<HashMap<RoleId, String>>(),
                ),
                Err(err) => FetchData::from_err(err),
            };

            Message::UpdateRoles(msg)
        });

        Self {
            entrant: FetchData::new(),
            roles: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Message) -> bool {
        match msg {
            Message::UpdateEntrant(entrant) => self.entrant = entrant,
            Message::UpdateRoles(roles) => self.roles = roles,
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.entrant.zip(&self.roles).render(|(entrant, roles)| {
            let title;

            let entrants = match &entrant.inner {
                EntrantVariant::Player(player) => {
                    title = player.name.clone();

                    html! {
                        <tr>
                            <td>{ player.name.clone() }</td>
                            <td>{ "It's a player!" }</td>
                        </tr>
                    }
                }
                EntrantVariant::Team(team) => {
                    title = team.name.clone();

                    team.players
                        .iter()
                        .map(|player| {
                            let role = match roles.get(&player.role) {
                                Some(role) => role.clone(),
                                None => "Unknown".to_string(),
                            };

                            html! {
                                <tr>
                                    <td>{ player.name.clone() }</td>
                                    <td>{ role }</td>
                                </tr>
                            }
                        })
                        .collect()
                }
            };

            html! {
                <div>
                    <h2 class="dt-title">{ title }</h2>
                    <table class="dt-table dt-table-center">
                        <tr>
                            <th>{ "Name" }</th>
                            <th>{ "Role" }</th>
                        </tr>
                        { entrants }
                    </table>
                </div>
            }
        })
    }
}

pub enum Message {
    UpdateEntrant(FetchData<ApiEntrant>),
    UpdateRoles(FetchData<HashMap<RoleId, String>>),
}
