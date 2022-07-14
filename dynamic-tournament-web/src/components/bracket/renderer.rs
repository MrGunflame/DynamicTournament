//! HTML renderer
use std::fmt::Display;

use dynamic_tournament_core::render::{
    self, BracketRound, BracketRounds, Position, Renderer, Round,
};
use dynamic_tournament_core::{EntrantScore, Match, Node, System};
use yew::{html, Context, Html};

use super::{Bracket, BracketMatch, Message};

pub struct HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    output: Html,
    ctx: &'a Context<Bracket>,
    tournament: &'a T,
}

impl<'a, T, E> HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    pub fn new(tournament: &'a T, ctx: &'a Context<Bracket>) -> Self {
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
        let on_action = self
            .ctx
            .link()
            .callback(move |action| Message::Action { index, action });

        let entrants = input
            .entrants
            .map(|e| e.map(|e| e.entrant(&self.tournament.borrow()).unwrap().clone()));

        let nodes = input.entrants.map(|e| e.map(|e| e.data));

        html! {
            <BracketMatch<E> {entrants} {nodes} {on_action} number={match_index} {position} />
        }
    }
}

impl<'a, T, E> Renderer<T, E, EntrantScore<u64>> for HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    fn render(&mut self, input: BracketRounds<'_, T>) {
        self.output = input
            .map(|bracket_round| self.render_bracket_round(bracket_round))
            .collect();
    }
}
