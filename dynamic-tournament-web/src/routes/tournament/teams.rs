use yew::prelude::*;
use yew_router::prelude::*;

use super::Route;

use dynamic_tournament_api::tournament as api;

use std::rc::Rc;

pub struct Teams;

impl Component for Teams {
    type Message = Message;
    type Properties = TeamsProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let id = ctx.props().teams.id;

        match msg {
            Message::OnClick(team_id) => {
                ctx.link()
                    .history()
                    .expect("No History given")
                    .push(Route::TeamDetails { id: id.0, team_id });

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let teams: Html = ctx
            .props()
            .teams
            .teams
            .iter()
            .enumerate()
            .map(|(i, team)| {
                let on_click = ctx.link().callback(move |_| Message::OnClick(i as u32));

                html! {
                    <tr onclick={on_click} class="tr-link">
                        <td>{ team.name.clone() }</td>
                        <td>{ team.players.len() }</td>
                    </tr>
                }
            })
            .collect();

        html! {
            <table class="table-center">
                <tbody>
                    <tr>
                        <th>{ "Name" }</th>
                        <th>{ "Players" }</th>
                    </tr>
                    {teams}
                </tbody>
            </table>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct TeamsProps {
    pub teams: Rc<api::Tournament>,
}

impl PartialEq for TeamsProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.teams, &other.teams)
    }
}

pub enum Message {
    OnClick(u32),
}
