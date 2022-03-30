use yew::prelude::*;
use yew_router::prelude::*;

use super::Route;

use crate::api::tournament as api;

use std::rc::Rc;

pub struct Teams;

impl Component for Teams {
    type Message = ();
    type Properties = TeamsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let teams: Html = ctx.props()
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
    pub teams: Rc<api::Tournament>,
}
