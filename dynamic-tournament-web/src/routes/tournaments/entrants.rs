mod entrant;
#[allow(clippy::module_inception)]
mod entrants;

use dynamic_tournament_api::v3::id::{EntrantId, TournamentId};
use yew::{html, Component, Context, Html, Properties};

use crate::utils::router::{Path, Routable, Switch};

#[derive(Copy, Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament_id: TournamentId,
}

#[derive(Debug)]
pub struct Entrants;

impl Component for Entrants {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tournament_id = ctx.props().tournament_id;

        html! {
            <Switch<Route> render={Switch::render(switch(tournament_id))} />
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Entrants,
    Entrant { id: EntrantId },
}

impl Routable for Route {
    fn from_path(path: &mut Path) -> Option<Self> {
        match path.take() {
            None => Some(Self::Entrants),
            Some(s) => match s.parse() {
                Ok(id) => Some(Self::Entrant { id }),
                Err(_) => None,
            },
        }
    }

    fn to_path(&self) -> String {
        match self {
            Self::Entrants => String::from("/"),
            Self::Entrant { id } => format!("/{}", id),
        }
    }
}

fn switch(tournament_id: TournamentId) -> impl Fn(&Route) -> Html {
    use entrant::Entrant;
    use entrants::Entrants;

    move |route| match route {
        Route::Entrants => html! {
            <Entrants {tournament_id} />
        },
        Route::Entrant { id } => html! {
            <Entrant {tournament_id} entrant_id={*id} />
        },
    }
}
