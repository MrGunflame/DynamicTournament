use yew::prelude::*;

use dynamic_tournament_api::tournament as api;

use std::rc::Rc;

pub struct TeamDetails;

impl Component for TeamDetails {
    type Message = ();
    type Properties = TeamDetailsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let team = match ctx
            .props()
            .teams
            .entrants
            .clone()
            .unwrap_teams()
            .get(ctx.props().index as usize)
        {
            Some(team) => team.clone(),
            None => {
                return html! {
                    <span>{"This Team does not exist! Unlucky"}</span>
                }
            }
        };

        let players: Html = team
            .players
            .into_iter()
            .map(|player| {
                html! {
                    <tr>
                        <td>{player.role.to_string()}</td>
                        <td>{player.account_name}</td>
                    </tr>
                }
            })
            .collect();

        html! {
            <div>
                <h2 class="title-label h-center">{team.name}</h2>
                <table class="table-center">
                    <tbody>
                        <tr>
                            <th>{"Role"}</th>
                            <th>{"Account Name"}</th>
                        </tr>
                        {players}
                    </tbody>
                </table>
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct TeamDetailsProps {
    pub teams: Rc<api::Tournament>,
    pub index: u32,
}

impl PartialEq for TeamDetailsProps {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && Rc::ptr_eq(&self.teams, &other.teams)
    }
}
