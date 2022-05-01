mod overview;
mod teamdetails;
mod teams;

use teamdetails::TeamDetails;
use teams::Teams;

use yew::prelude::*;
use yew_router::prelude::*;

use crate::components::bracket::Bracket;
use crate::components::movable_boxed::MovableBoxed;
use crate::utils::FetchData;
use crate::{DataResult, Title};

use dynamic_tournament_api::tournament as api;
use dynamic_tournament_api::tournament::TournamentId;
use dynamic_tournament_api::Client;

use std::rc::Rc;

use overview::Overview;

pub struct Tournament {
    tournament: FetchData<Rc<api::Tournament>>,
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
            ) -> DataResult<Rc<api::Tournament>> {
                let client = client.tournaments();

                let data = client.get(id).await?;

                Ok(Rc::new(data))
            }

            let data = Some(fetch_data(client, id).await);

            Msg::Update(data.into())
        });

        Self {
            tournament: FetchData::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(data) => {
                self.tournament = data;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.tournament.render(|tournament| {
            Title::set(&tournament.name);

            let tournament = tournament.clone();

            let id = ctx.props().id.0;

            let switch = move |route: &Route| -> Html {
                let tournament = tournament.clone();

                let mut routes = Vec::with_capacity(4);
                for (r, n) in &[
                    (Route::Index { id }, "Overview"),
                    (Route::Bracket { id }, "Bracket"),
                    (Route::Teams { id }, "Teams"),
                ] {
                    let classes = if r == route { "active" } else { "" };

                    routes.push(html! {
                        <li><Link<Route> classes={classes} to={r.clone()}>{ n }</Link<Route>></li>
                    });
                }

                routes.push(html! {
                    <li><Link<crate::routes::Route> to={crate::routes::Route::Embed { id: tournament.id.0 }}>{ "Embed Mode" }</Link<crate::routes::Route>></li>
                });

                let content = match route {
                    Route::Index { id: _ } => html! {
                        <Overview tournament={tournament.clone()} />
                    },
                    Route::Bracket { id: _ } => html! {
                        <MovableBoxed>
                            <Bracket tournament={tournament.clone()} />
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
                            <i aria-hidden="true" class="fa-solid fa-angle-left"></i>
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
    Update(FetchData<Rc<api::Tournament>>),
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
