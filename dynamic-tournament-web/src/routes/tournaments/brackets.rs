mod bracket;
#[allow(clippy::module_inception)]
mod brackets;

use dynamic_tournament_api::v3::id::BracketId;
use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::{html, Component, Context, Html, Properties};

use crate::utils::router::{Path, Routable, Switch};
use crate::utils::Rc;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

#[derive(Debug)]
pub struct Brackets;

impl Component for Brackets {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tournament = ctx.props().tournament.clone();

        html! {
            <Switch<Route> render={Switch::render(switch(tournament))} />
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Route {
    Brackets,
    Bracket { id: BracketId },
}

impl Routable for Route {
    fn from_path(path: &mut Path) -> Option<Self> {
        match path.take() {
            None => Some(Self::Brackets),
            Some(s) => match s.parse() {
                Ok(id) => Some(Self::Bracket { id }),
                Err(_) => None,
            },
        }
    }

    fn to_path(&self) -> String {
        match self {
            Self::Brackets => String::from("/"),
            Self::Bracket { id } => format!("/{}", id),
        }
    }
}

fn switch(tournament: Rc<Tournament>) -> impl Fn(&Route) -> Html {
    use bracket::Bracket;
    use brackets::Brackets;

    move |route| {
        let tournament = tournament.clone();

        match route {
            Route::Brackets => html! {
                <Brackets {tournament} />
            },
            Route::Bracket { id } => html! {
                <Bracket {tournament} bracket_id={*id} />
            },
        }
    }
}
