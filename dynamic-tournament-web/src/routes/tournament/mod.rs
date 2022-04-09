mod teamdetails;
mod teams;

use teamdetails::TeamDetails;
use teams::Teams;

use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::bracket::Bracket;
use crate::{render_data, Data, DataResult};

use dynamic_tournament_api::tournament as api;
use dynamic_tournament_api::tournament::TournamentId;
use dynamic_tournament_api::Client;

use std::rc::Rc;

pub struct Tournament {
    data: Data<(Rc<api::Tournament>, Option<Rc<api::Bracket>>)>,
}

impl Component for Tournament {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let (client, _) = ctx.link().context::<Client>(Callback::noop()).unwrap();

        let id = ctx.props().id;
        link.send_future(async move {
            async fn fetch_data(
                client: Client,
                id: TournamentId,
            ) -> DataResult<(Rc<api::Tournament>, Option<Rc<api::Bracket>>)> {
                let client = client.tournaments();

                let data = client.get(id).await?;

                let bracket = match client.bracket(id).get().await {
                    Ok(bracket) => Some(Rc::new(bracket)),
                    Err(_) => None,
                };

                Ok((Rc::new(data), bracket))
            }

            let data = Some(fetch_data(client, id).await);

            Msg::Update(data)
        });

        Self { data: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(data) => {
                self.data = data;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        render_data(&self.data, |(data, bracket)| {
            let tournament = data.clone();
            let bracket = bracket.clone();

            let switch = move |route: &Route| -> Html {
                let tournament = tournament.clone();
                let bracket = bracket.clone();

                match route {
                    Route::Index { id } => html! {
                        <span>{ format!("Tournament id {}", id) }</span>
                    },
                    Route::Bracket { id: _ } => html! {
                        <Bracket tournament={tournament} bracket={bracket} />
                    },
                    Route::Teams { id: _ } => html! {
                        <Teams teams={tournament} />
                    },
                    Route::TeamDetails { id: _, team_id } => html! {
                        <TeamDetails teams={tournament} index={*team_id} />
                    },
                }
            };

            html! {
                <>
                    <div class="navbar">
                        <ul>
                            <li><Link<Route> to={Route::Bracket{ id: ctx.props().id.0 }}>{ "Bracket" }</Link<Route>></li>
                            <li><Link<Route> to={Route::Teams{ id: ctx.props().id.0 }}>{ "Teams" }</Link<Route>></li>
                        </ul>
                    </div>
                    <Switch<Route> render={Switch::render(switch)} />
                </>
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub id: TournamentId,
}

pub enum Msg {
    Update(Data<(Rc<api::Tournament>, Option<Rc<api::Bracket>>)>),
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/tournament/:id")]
    Index { id: u64 },
    #[at("/tournament/:id/bracket")]
    Bracket { id: u64 },
    #[at("/tournament/:id/teams")]
    Teams { id: u64 },
    #[at("/tournament/:id/teams/:team_id")]
    TeamDetails { id: u64, team_id: u32 },
}
