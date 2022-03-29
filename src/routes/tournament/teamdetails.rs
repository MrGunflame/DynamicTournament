use yew::prelude::*;

use std::rc::Rc;

pub struct TeamDetails {
    teams: Rc<crate::MatchmakerInput>,
    index: u32,
}

impl Component for TeamDetails {
    type Message = ();
    type Properties = TeamDetailsProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            teams: ctx.props().teams.clone(),
            index: ctx.props().index,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let team = match self.teams.teams.get(self.index as usize) {
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
                        <td>{player.role}</td>
                        <td>{player.account_name}</td>
                    </tr>
                }
            })
            .collect();

        html! {
            <div>
                <h2>{team.name}</h2>
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

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct TeamDetailsProps {
    pub teams: Rc<crate::MatchmakerInput>,
    pub index: u32,
}
