use dynamic_tournament_api::v3::id::TournamentId;
use dynamic_tournament_api::v3::tournaments::Tournament as ApiTournament;
use yew::{html, Component, Context, Html, Properties};

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::router::{Link, Path, Routable, Switch};
use crate::utils::{FetchData, Rc};

use super::{Admin, Brackets, Entrants, Overview};

#[derive(Copy, Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub id: TournamentId,
}

#[derive(Debug)]
pub struct Tournament {
    tournament: FetchData<Rc<ApiTournament>>,
    is_admin: bool,
}

impl Component for Tournament {
    type Message = FetchData<Rc<ApiTournament>>;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let is_admin = client.is_authenticated();

        let id = ctx.props().id;
        ctx.link().send_future(async move {
            match client.v3().tournaments().get(id).await {
                Ok(val) => FetchData::from(Rc::new(val)),
                Err(err) => FetchData::from_err(err),
            }
        });

        Self {
            tournament: FetchData::new(),
            is_admin,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: FetchData<Rc<ApiTournament>>) -> bool {
        self.tournament = msg;
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.tournament.render(|tournament| {
            let tournament = tournament.clone();
            let is_admin = self.is_admin;

            html! {
                <Switch<Route> render={Switch::render(switch(tournament, is_admin))} />
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
    fn from_path(path: &mut Path) -> Option<Self> {
        match path.take() {
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

fn switch(tournament: Rc<ApiTournament>, is_admin: bool) -> impl Fn(&Route) -> Html {
    move |route| {
        let mut links = vec![
            (Route::Index, "Overview"),
            (Route::Brackets, "Brackets"),
            (Route::Entrants, "Entrants"),
        ];

        if is_admin {
            links.push((Route::Admin, "Admin"));
        }

        let routes: Html = links
            .into_iter()
            .map(|(r, name)| {
                let classes = if r == *route { "active" } else { "" };

                let to = format!("/tournaments/{}{}", tournament.id, r.to_path());

                html! {
                    <li><Link {classes} {to}>{ name }</Link></li>
                }
            })
            .collect();

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
                <Link classes="link-inline link-back" to={"/tournaments"}>
                    <i aria-hidden="true" class="fa-solid fa-angle-left"></i>
                    { "Back to Tournaments" }
                </Link>

                <h2 class="tournament-name">{ tournament.name.clone() }</h2>
                <div class="navbar">
                    <ul>
                        { routes }
                    </ul>
                </div>
                { content }
            </>
        }
    }
}
