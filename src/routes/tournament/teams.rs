use yew::callback::Callback;
use yew::prelude::*;
use yew_router::prelude::*;

use super::Route;

use std::rc::Rc;

pub struct Teams {
    teams: Rc<crate::MatchmakerInput>,
}

impl Component for Teams {
    type Message = ();
    type Properties = TeamsProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            teams: ctx.props().teams.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let teams: Html = self
            .teams
            .teams
            .iter()
            .enumerate()
            .map(|(i, team)| {
                html! {
                    <tr>
                        <td>{ i }</td>
                        <td>{ team.name.clone() }</td>
                        <td><Link<Route> classes="link-inline" to={Route::TeamDetails { id: i as u32} }>{"Details"}</Link<Route>></td>
                    </tr>
                }
            })
            .collect();

        html! {
            <table class="table-center">
                <tbody>
                    <tr>
                        <th>{"ID"}</th>
                        <th>{"Name"}</th>
                    </tr>
                    {teams}
                </tbody>
            </table>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct TeamsProps {
    pub teams: Rc<crate::MatchmakerInput>,
}
