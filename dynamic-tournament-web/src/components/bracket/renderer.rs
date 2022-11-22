//! HTML renderer
use std::fmt::Display;

use dynamic_tournament_core::render::{Element, ElementKind, Position, Renderer};
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
    fn render_element<'b>(&mut self, elem: Element<'b, T>) -> Html {
        match elem.kind() {
            ElementKind::Container => self.render_container(elem),
            ElementKind::Row => self.render_row(elem),
            ElementKind::Column => self.render_column(elem),
            ElementKind::Match => self.render_match(elem, 0),
        }
    }

    fn render_container<'b>(&mut self, elem: Element<'b, T>) -> Html {
        let inner = elem.inner.unwrap_container();
        self.render_element(*inner.into_inner())
    }

    fn render_column<'b>(&mut self, column: Element<'b, T>) -> Html {
        let column = column.inner.unwrap_column();
        let inner = self.render_iter(column);

        html! {
            <div class="dt-bracket-column">
                { inner }
            </div>
        }
    }

    fn render_row<'b>(&mut self, row: Element<'b, T>) -> Html {
        let inner = self.render_iter(row.inner.unwrap_row());

        html! {
            <div class="dt-bracket-row">
                { inner }
            </div>
        }
    }

    fn render_iter<'b, I>(&mut self, iter: I) -> Html
    where
        I: Iterator<Item = Element<'b, T>>,
        T: 'b,
    {
        iter.enumerate()
            .map(|(index, elem)| match elem.kind() {
                ElementKind::Container => self.render_container(elem),
                ElementKind::Row => self.render_row(elem),
                ElementKind::Column => self.render_column(elem),
                ElementKind::Match => self.render_match(elem, index),
            })
            .collect()
    }

    fn render_match(&self, m: Element<'_, T>, round_index: usize) -> Html {
        let inner = m.inner.unwrap_match();

        // Get the match from the tournament.
        let match_: &Match<Node<EntrantScore<u64>>> =
            unsafe { self.tournament.matches().get_unchecked(inner.index()) };

        let entrants = match_.map(|spot| {
            spot.map(|node| {
                // SAFE
                unsafe { self.tournament.entrants().get_unchecked(node.index).clone() }
            })
        });

        let nodes = match_.map(|spot| spot.map(|node| node.data));

        let position = m.position.unwrap_or(Position::SpaceAround);

        let index = inner.index();
        let on_action = self
            .ctx
            .link()
            .callback(move |action| Message::Action { index, action });

        html! {
            <BracketMatch<E> {entrants} {nodes} {on_action} number={round_index + 1} {position} />
        }
    }
}

impl<'a, T, E> Renderer<T, E, EntrantScore<u64>> for HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
    Self: 'a,
{
    fn render(&mut self, input: Element<'_, T>) {
        self.output = self.render_element(input);
    }
}
