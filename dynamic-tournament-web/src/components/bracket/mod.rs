pub mod double_elimination;
pub mod r#match;
pub mod single_elimination;
pub mod team;

pub use double_elimination::DoubleEliminationBracket;
pub use r#match::{Action, BracketMatch};
pub use single_elimination::SingleEliminationBracket;
pub use team::BracketTeam;

use dynamic_tournament_api::tournament::{Bracket as BracketState, BracketType, Tournament};

use yew::prelude::*;

use std::rc::Rc;

pub enum Bracket {
    SingleElimination,
    DoubleElimination,
}

impl Component for Bracket {
    type Message = ();
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        match ctx.props().tournament.bracket_type {
            BracketType::SingleElimination => Self::SingleElimination,
            BracketType::DoubleElimination => Self::DoubleElimination,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let tournament = ctx.props().tournament.clone();
        let bracket = ctx.props().bracket.clone();

        match self {
            Self::SingleElimination => html! {
                <SingleEliminationBracket tournament={tournament} bracket={bracket} />
            },
            Self::DoubleElimination => html! {
                <DoubleEliminationBracket tournament={tournament} bracket={bracket} />
            },
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub tournament: Rc<Tournament>,
    pub bracket: Option<Rc<BracketState>>,
}

impl PartialEq for Properties {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
            && self
                .bracket
                .as_ref()
                .zip(other.bracket.as_ref())
                .map_or(false, |(a, b)| Rc::ptr_eq(a, b))
    }
}