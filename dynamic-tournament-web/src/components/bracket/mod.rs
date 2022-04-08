pub mod double_elimination;
pub mod r#match;
pub mod single_elimination;
pub mod team;

pub use double_elimination::DoubleEliminationBracket;
pub use r#match::{Action, BracketMatch};
pub use single_elimination::SingleEliminationBracket;
pub use team::BracketTeam;

use crate::api::tournament::{BracketType, Tournament};
use crate::api::v1::tournament as api;

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
    pub bracket: Option<Rc<api::Bracket>>,
}

impl PartialEq for Properties {
    fn eq(&self, other: &Self) -> bool {
        if !Rc::ptr_eq(&self.tournament, &other.tournament) {
            return false;
        }

        if !self.bracket.is_some() && other.bracket.is_some() {
            return false;
        }

        // FIXME: Use unwrap unchecked.
        let this = self.bracket.as_ref().unwrap();
        let other = other.bracket.as_ref().unwrap();

        Rc::ptr_eq(this, other)
    }
}
