//! HTML renderer
use std::fmt::Display;

use dynamic_tournament_core::render::{
    Column, Container, ContainerIter, MatchesIter, Renderer, Row,
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
    fn render_column(&mut self, column: &Column<'_, T>) -> Html {
        let inner = match column.iter() {
            ContainerIter::Columns(cols) => cols.map(|col| self.render_column(col)).collect(),
            ContainerIter::Rows(rows) => rows.map(|row| self.render_row(row)).collect(),
            ContainerIter::Matches(matches) => self.render_matches(matches),
        };

        html! {
            <div class="dt-bracket-column">
                { inner }
            </div>
        }
    }

    fn render_row(&mut self, row: &Row<'_, T>) -> Html {
        let inner = match row.iter() {
            ContainerIter::Columns(cols) => cols.map(|col| self.render_column(col)).collect(),
            ContainerIter::Rows(rows) => rows.map(|row| self.render_row(row)).collect(),
            ContainerIter::Matches(matches) => self.render_matches(matches),
        };

        html! {
            <div class="dt-bracket-row">
                { inner }
            </div>
        }
    }

    fn render_matches(&mut self, matches: MatchesIter<'_, T>) -> Html {
        matches
            .enumerate()
            .map(|(i, m)| {
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

                let position = m.position();

                let index = m.index();
                let on_action = self
                    .ctx
                    .link()
                    .callback(move |action| Message::Action { index, action });

                html! {
                    <BracketMatch<E> {entrants} {nodes} {on_action} number={i + 1} {position} />
                }
            })
            .collect()
    }
}

impl<'a, T, E> Renderer<T, E, EntrantScore<u64>> for HtmlRenderer<'a, T, E>
where
    T: System<Entrant = E, NodeData = EntrantScore<u64>>,
    E: Clone + Display + 'static,
{
    fn render(&mut self, input: Container<'_, T>) {
        self.output = match input.iter() {
            ContainerIter::Columns(cols) => cols.map(|col| self.render_column(col)).collect(),
            ContainerIter::Rows(rows) => rows.map(|row| self.render_row(row)).collect(),
            ContainerIter::Matches(matches) => self.render_matches(matches),
        };
    }
}
