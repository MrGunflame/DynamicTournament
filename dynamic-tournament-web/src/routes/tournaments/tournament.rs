use dynamic_tournament_api::v3::id::TournamentId;
use dynamic_tournament_api::v3::tournaments::Tournament as ApiTournament;
use yew::{html, Component, Context, Html, Properties};

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::router::{Link, PathBuf, Routable, Switch};
use crate::utils::{FetchData, Rc};

use super::navbar::Navbar;
use super::{Admin, Brackets, Entrants, Overview};
use crate::components::icons::FaAngleLeft;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub id: TournamentId,
}

#[derive(Debug)]
pub struct Tournament {
    tournament: FetchData<Rc<ApiTournament>>,
}

impl Component for Tournament {
    type Message = FetchData<Rc<ApiTournament>>;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let id = ctx.props().id;
        ctx.link().send_future(async move {
            match client.v3().tournaments().get(id).await {
                Ok(val) => FetchData::from(Rc::new(val)),
                Err(err) => FetchData::from_err(err),
            }
        });

        Self {
            tournament: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: FetchData<Rc<ApiTournament>>) -> bool {
        self.tournament = msg;
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.tournament.render(|tournament| {
            let tournament = tournament.clone();

            html! {
                <Switch<Route> render={Switch::render(switch(tournament))} />
            }
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Index,
    Brackets,
    Entrants,
    Admin,
}

impl Routable for Route {
    fn from_path(path: &mut PathBuf) -> Option<Self> {
        match path.take().as_deref() {
            None => Some(Self::Index),
            Some("brackets") => Some(Self::Brackets),
            Some("entrants") => Some(Self::Entrants),
            Some("admin") => Some(Self::Admin),
            Some(_) => None,
        }
    }

    fn to_path(&self) -> String {
        match self {
            Self::Index => String::from("/"),
            Self::Brackets => String::from("/brackets"),
            Self::Entrants => String::from("/entrants"),
            Self::Admin => String::from("/admin"),
        }
    }
}

fn switch(tournament: Rc<ApiTournament>) -> impl Fn(&Route) -> Html {
    move |route| {
        let content = {
            let tournament = tournament.clone();

            match route {
                Route::Index => html! {
                    <Overview {tournament} />
                },
                Route::Brackets => html! {
                    <Brackets {tournament} />
                },
                Route::Entrants => html! {
                    <Entrants tournament_id={tournament.id} />
                },
                Route::Admin => html! {
                    <Admin {tournament} />
                },
            }
        };

        html! {
            <>
                <Link classes="link-inline link-back" to={"/"}>
                    <FaAngleLeft label="Back" />
                    { "Back to Tournaments" }
                </Link>

                <h2 class="tournament-name">{ tournament.name.clone() }</h2>
                <Navbar tournament_id={tournament.id} route={*route} />
                { content }
            </>
        }
    }
}
