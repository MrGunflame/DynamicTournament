mod teamdetails;
mod teams;

use teamdetails::TeamDetails;
use teams::Teams;

use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::bracket::Bracket;
use crate::components::movable_boxed::MovableBoxed;
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

            let id = ctx.props().id.0;

            let switch = move |route: &Route| -> Html {
                let tournament = tournament.clone();
                let bracket = bracket.clone();

                let mut routes = Vec::with_capacity(2);
                for (r, n) in &[
                    (Route::Bracket { id }, "Bracket"),
                    (Route::Teams { id }, "Teams"),
                ] {
                    let classes = if r == route { "active" } else { "" };

                    routes.push(html! {
                        <li><Link<Route> classes={classes} to={r.clone()}>{ n }</Link<Route>></li>
                    });
                }

                let content = match route {
                    Route::Index { id } => html! {
                        <span>{ format!("Tournament id {}", id) }</span>
                    },
                    Route::Bracket { id: _ } => html! {
                        <MovableBoxed>
                            <Bracket tournament={tournament.clone()} bracket={bracket} />
                        </MovableBoxed>
                    },
                    Route::Teams { id: _ } => html! {
                        <Teams teams={tournament.clone()} />
                    },
                    Route::TeamDetails { id: _, team_id } => html! {
                        <TeamDetails teams={tournament.clone()} index={*team_id} />
                    },
                };

                html! {
                    <>
                        <Link<crate::routes::Route> classes="link-inline link-back" to={crate::routes::Route::TournamentList}>
                            <i class="fa-solid fa-angle-left"></i>
                            { "Back to Tournaments" }
                        </Link<crate::routes::Route>>
                        <h2 class="tournament-name">{ tournament.name.clone() }</h2>
                        <div class="navbar">
                            <ul>
                                {routes}
                            </ul>
                        </div>
                        {content}
                    </>
                }
            };

            html! {
                <>
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
