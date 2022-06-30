use std::{fmt::Display, rc::Rc};

use dynamic_tournament_generator::{
    options::TournamentOptions,
    render::{self, BracketRound, Position, Renderer, Round},
    tournament::{Tournament, TournamentKind},
    EntrantScore, Match, Node, System,
};
use web_sys::MouseEvent;
use yew::{html, Callback, Component, Context, Html, Properties};

use dynamic_tournament_api::v3::{
    id::{EntrantId, SystemId},
    tournaments::brackets::Bracket,
};

use super::r#match::BracketMatch;

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub bracket: Rc<Bracket>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.bracket, &other.bracket)
    }
}

/// Similar to the bracket seen by the user, but without any actual match data.
///
/// Instead this allows organizing the bracket by moving entrants between spots.
pub struct AdminBracket {
    entrants: Vec<EntrantId>,
    state: Tournament<EntrantId, EntrantScore<u64>>,
}

impl Component for AdminBracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let kind = match ctx.props().bracket.system {
            SystemId(1) => TournamentKind::SingleElimination,
            SystemId(2) => TournamentKind::DoubleElimination,
            _ => panic!(),
        };

        let mut state = Tournament::new(kind, TournamentOptions::default());
        state.extend(ctx.props().bracket.entrants.iter().cloned());

        Self {
            entrants: ctx.props().bracket.entrants.clone(),
            state,
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let bracket = HtmlRenderer::new(&self.state, ctx).into_output();

        html! {
            { bracket }
        }
    }
}

pub enum Message {
    Attach(usize),
    Move { x: u32, y: u32 },
    Release(usize),
}

pub struct HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    output: Html,
    ctx: &'a Context<AdminBracket>,
    tournament: &'a T,
}

impl<'a, T, E> HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    pub fn new(tournament: &'a T, ctx: &'a Context<AdminBracket>) -> Self {
        Self {
            output: html! {},
            ctx,
            tournament,
        }
    }

    pub fn into_output(mut self) -> Html {
        self.tournament.render(&mut self);

        html! {
            <div class="dt-bracket">
                { self.output }
            </div>
        }
    }
}

impl<'a, T, E> HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    fn render_bracket_round(&self, input: BracketRound<'_, T>) -> Html {
        let brackets: Html = input.map(|bracket| self.render_bracket(bracket)).collect();

        html! {
            <div class="dt-bracket-bracket-row">
                { brackets }
            </div>
        }
    }

    fn render_bracket(&self, input: render::Bracket<'_, T>) -> Html {
        let rounds: Html = input.map(|round| self.render_round(round)).collect();

        html! {
            <div class="dt-bracket-bracket">
                { rounds }
            </div>
        }
    }

    fn render_round(&self, input: Round<'_, T>) -> Html {
        let round: Html = input
            .indexed()
            .enumerate()
            .map(|(match_index, (index, m, pos))| {
                html! {
                    { self.render_match(m, index, match_index.saturating_add(1), pos) }
                }
            })
            .collect();

        html! {
            <div class="dt-bracket-round">
                { round }
            </div>
        }
    }

    fn render_match(
        &self,
        input: &Match<Node<EntrantScore<u64>>>,
        index: usize,
        match_index: usize,
        position: Position,
    ) -> Html {
        let entrants = input
            .entrants
            .map(|e| e.map(|e| e.entrant(&self.tournament.borrow()).unwrap().clone()));

        // The draggable container.
        html! {
            <div class="dt-bracket-drag">
                <BracketMatch<E> {entrants} {position} />
            </div>
        }
    }
}

impl<'a, T, E> Renderer<T, E, EntrantScore<u64>> for HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    fn render(&mut self, input: dynamic_tournament_generator::render::BracketRounds<'_, T>) {
        self.output = input
            .map(|bracket_round| self.render_bracket_round(bracket_round))
            .collect();
    }
}
