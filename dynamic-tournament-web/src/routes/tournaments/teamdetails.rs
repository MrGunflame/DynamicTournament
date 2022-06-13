use std::collections::HashMap;

use yew::prelude::*;

use dynamic_tournament_api::v3::{
    id::{EntrantId, RoleId, TournamentId},
    tournaments::{
        entrants::{Entrant, EntrantVariant},
        roles::Role,
    },
};
use dynamic_tournament_api::Client;

use crate::utils::FetchData;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament_id: TournamentId,
    pub id: EntrantId,
}

#[derive(Debug)]
pub struct TeamDetails {
    entrant: FetchData<Entrant>,
    roles: FetchData<HashMap<RoleId, Role>>,
}

impl Component for TeamDetails {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("no client in context");

        let tournament_id = ctx.props().tournament_id;
        let id = ctx.props().id;

        {
            let client = client.clone();
            ctx.link().send_future(async move {
                let msg = match client
                    .v3()
                    .tournaments()
                    .entrants(tournament_id)
                    .get(id)
                    .await
                {
                    Ok(entrant) => FetchData::from(entrant),
                    Err(err) => FetchData::from_err(err),
                };

                Message::UpdateEntrant(msg)
            });
        }

        ctx.link().send_future(async move {
            let msg = match client.v3().tournaments().roles(tournament_id).list().await {
                Ok(roles) => {
                    // Convert the Vec into a HashMap with the ids as key.
                    let roles: HashMap<RoleId, Role> =
                        roles.into_iter().map(|role| (role.id, role)).collect();

                    FetchData::from(roles)
                }
                Err(err) => FetchData::from_err(err),
            };

            Message::UpdateRoles(msg)
        });

        Self {
            entrant: FetchData::new(),
            roles: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateEntrant(entrant) => self.entrant = entrant,
            Message::UpdateRoles(roles) => self.roles = roles,
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        if !self.roles.has_value() {
            return self.roles.render(|_| html! {});
        }

        self.entrant.render(|entrant| {
            let title;

            let roles = self.roles.as_ref().unwrap();

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
                                Some(role) => role.name.clone(),
                                None => String::from("Unknown"),
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
                    <h2 class="title-label h-center">{ title }</h2>
                    <table class="table-center">
                        <tbody>
                            <tr>
                                <th>{ "Name" }</th>
                                <th>{ "Role" }</th>
                            </tr>
                            { entrants }
                        </tbody>
                    </table>
                </div>
            }
        })
    }
}

pub enum Message {
    UpdateEntrant(FetchData<Entrant>),
    UpdateRoles(FetchData<HashMap<RoleId, Role>>),
}
