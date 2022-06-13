mod brackets;
mod entrants;
mod overview;
mod teamdetails;

use entrants::Entrants;
use teamdetails::TeamDetails;

use yew::prelude::*;
use yew_router::prelude::*;

use self::brackets::bracket::Bracket;
use self::brackets::Brackets;

use crate::utils::FetchData;
use crate::Title;

use dynamic_tournament_api::Client;

use dynamic_tournament_api::v3::id::{BracketId, EntrantId, TournamentId};
use dynamic_tournament_api::v3::tournaments::Tournament as ApiTournament;

use std::rc::Rc;

use overview::Overview;

pub struct Tournament {
    tournament: FetchData<Rc<ApiTournament>>,
}

impl Component for Tournament {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("no client in context");

        let id = ctx.props().id;
        link.send_future(async move {
            let tournament = match client.v3().tournaments().get(id).await {
                Ok(tournament) => FetchData::from(Rc::new(tournament)),
                Err(err) => FetchData::from_err(err),
            };

            Msg::Update(tournament)
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

            let tournament_id = ctx.props().id;

            let switch = move |route: &Route| -> Html {
                let tournament = tournament.clone();

                let tournament_name = tournament.name.clone();

                let mut routes = Vec::with_capacity(3);
                for (r, n) in &[
                    (Route::Index { tournament_id, tournament_name: tournament_name.clone() }, "Overview"),
                    (Route::Brackets { tournament_id, tournament_name: tournament_name.clone() }, "Brackets"),
                    (Route::Teams { tournament_id, tournament_name }, "Entrants"),
                ] {
                    let classes = if r == route { "active" } else { "" };

                    routes.push(html! {
                        <li><Link<Route> classes={classes} to={r.clone()}>{ n }</Link<Route>></li>
                    });
                }

                let content = match route {
                    Route::Index { tournament_id: _,tournament_name:_ } => html! {
                        <Overview tournament={tournament.clone()} />
                    },
                    Route::Brackets { tournament_id: _, tournament_name: _ } => html! {
                        <Brackets tournament={tournament.clone()} />
                    },
                    Route::Bracket{ tournament_id: _, tournament_name: _, bracket_id, bracket_name: _ }=> html! {
                        <Bracket tournament={tournament.clone()} id={*bracket_id} />
                    },
                    Route::Teams { tournament_id: _, tournament_name: _, } => html! {
                        <Entrants tournament={ tournament.clone() } />
                    },
                    Route::TeamDetails { tournament_id: _, tournament_name: _, team_id } => html! {
                        <TeamDetails {tournament_id} id={*team_id} />
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
    Update(FetchData<Rc<ApiTournament>>),
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/tournaments/:tournament_id/:tournament_name")]
    Index {
        tournament_id: TournamentId,
        tournament_name: String,
    },
    #[at("/tournaments/:tournament_id/:tournament_name/brackets")]
    Brackets {
        tournament_id: TournamentId,
        tournament_name: String,
    },
    #[at("/tournaments/:tournament_id/:tournament_name/brackets/:bracket_id/:bracket_name")]
    Bracket {
        tournament_id: TournamentId,
        tournament_name: String,
        bracket_id: BracketId,
        bracket_name: String,
    },
    #[at("/tournaments/:tournament_id/:tournament_name/entrants")]
    Teams {
        tournament_id: TournamentId,
        tournament_name: String,
    },
    #[at("/tournaments/:tournament_id/:tournament_name/entrants/:team_id")]
    TeamDetails {
        tournament_id: TournamentId,
        tournament_name: String,
        team_id: EntrantId,
    },
}
