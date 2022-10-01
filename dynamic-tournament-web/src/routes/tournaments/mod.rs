mod admin;
mod brackets;
mod entrants;
mod navbar;
mod overview;
mod teamdetails;
mod tournament;

use entrants::Entrants;

use yew::prelude::*;

use self::admin::Admin;
use self::brackets::Brackets;

use crate::utils::router::{PathBuf, Routable, Switch};

use dynamic_tournament_api::v3::id::TournamentId;

pub use self::tournament::Tournament;
use overview::Overview;

pub struct Tournaments;

impl Component for Tournaments {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <Switch<Route> render={Switch::render(switch)} />
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Props {
    pub id: TournamentId,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Route {
    Index,
    Tournament { id: TournamentId },
}

impl Routable for Route {
    fn from_path(path: &mut PathBuf) -> Option<Self> {
        match path.take() {
            None => Some(Self::Index),
            Some(s) => {
                let id = s.parse().ok()?;

                Some(Self::Tournament { id })
            }
        }
    }

    fn to_path(&self) -> String {
        match self {
            Route::Index => String::from("/"),
            Route::Tournament { id } => format!("/{}", id),
        }
    }
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Index => html! {
            <super::tournamentlist::TournamentList />
        },
        Route::Tournament { id } => html! {
            <Tournament id={*id} />
        },
    }
}
