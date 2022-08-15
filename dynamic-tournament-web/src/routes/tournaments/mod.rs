mod admin;
mod brackets;
mod entrants;
mod overview;
mod teamdetails;
mod tournament;

use entrants::Entrants;

use yew::prelude::*;

use self::admin::Admin;
use self::brackets::Brackets;

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::router::{Path, Routable, Switch};
use crate::utils::{FetchData, Rc};
use crate::Title;

use dynamic_tournament_api::v3::id::TournamentId;
use dynamic_tournament_api::v3::tournaments::Tournament as ApiTournament;

use overview::Overview;

pub struct Tournaments;

impl Component for Tournaments {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <Switch<Route> render={Switch::render(switch)} />
        }
    }
}

pub struct Tournament {
    tournament: FetchData<Rc<ApiTournament>>,
}

impl Component for Tournament {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let client = ClientProvider::get(ctx);

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

#[derive(Clone, PartialEq)]
pub enum Route {
    Index,
    Tournament { id: TournamentId, name: String },
}

impl Routable for Route {
    fn from_path(path: &mut Path) -> Option<Self> {
        match path.take() {
            None => Some(Self::Index),
            Some(s) => {
                let id = s.parse().ok()?;
                let name = path.take()?.to_string();

                Some(Self::Tournament { id, name })
            }
        }
    }

    fn to_path(&self) -> String {
        match self {
            Route::Index => String::from("/"),
            Route::Tournament { id, name } => format!("{}/{}", id, name),
        }
    }
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Index => html! {
            <super::tournamentlist::TournamentList />
        },
        Route::Tournament { id, name: _ } => html! {
            <Tournament id={*id} />
        },
    }
}
