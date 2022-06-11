use yew::prelude::*;

use dynamic_tournament_api::v3::{
    id::{EntrantId, TournamentId},
    tournaments::entrants::{Entrant, EntrantVariant},
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
}

impl Component for TeamDetails {
    type Message = FetchData<Entrant>;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("no client in context");

        let tournament_id = ctx.props().tournament_id;
        let id = ctx.props().id;
        ctx.link().send_future(async move {
            match client
                .v3()
                .tournaments()
                .entrants(tournament_id)
                .get(id)
                .await
            {
                Ok(entrant) => FetchData::from(entrant),
                Err(err) => FetchData::from_err(err),
            }
        });

        Self {
            entrant: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.entrant = msg;
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.entrant.render(|entrant| {
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
                            html! {
                                <tr>
                                    <td>{ player.name.clone() }</td>
                                    <td>{ player.role.to_string() }</td>
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
