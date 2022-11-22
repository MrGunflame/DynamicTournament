//! HTML renderer
use std::fmt::Display;

use dynamic_tournament_core::render::{self, Column, Element, Position, Renderer, Row};
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
        match elem {
            Element::Row(elem) => self.render_row(elem),
            Element::Column(elem) => self.render_column(elem),
            Element::Match(elem) => self.render_match(elem, 0),
        }
    }

    fn render_column<'b>(&mut self, column: Column<'b, T>) -> Html {
        let inner = self.render_iter(column);

        html! {
            <div class="dt-bracket-column">
                { inner }
            </div>
        }
    }

    fn render_row<'b>(&mut self, row: Row<'b, T>) -> Html {
        let inner = self.render_iter(row);

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
            .map(|(index, elem)| match elem {
                Element::Row(elem) => self.render_row(elem),
                Element::Column(elem) => self.render_column(elem),
                Element::Match(elem) => self.render_match(elem, index),
            })
            .collect()
    }

    fn render_match(&self, m: render::Match<'_, T>, round_index: usize) -> Html {
        // Get the match from the tournament.
        let match_: &Match<Node<EntrantScore<u64>>> =
            unsafe { self.tournament.matches().get_unchecked(m.index()) };

        let entrants = match_.map(|spot| {
            spot.map(|node| {
                // SAFE
                unsafe { self.tournament.entrants().get_unchecked(node.index).clone() }
            })
        });

        let nodes = match_.map(|spot| spot.map(|node| node.data));

        let position = m.position.unwrap_or(Position::SpaceAround);

        let index = m.index();
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
