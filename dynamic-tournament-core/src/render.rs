//! # Tournament Rendering
//!
//! The `render` module provides types to generically render tournament [`System`]s.
use crate::{Match, Node, System};

use std::ops::Range;

/// A renderer used to render any [`System`].
pub trait Renderer<T, E, D>
where
    T: System<Entrant = E, NodeData = D>,
{
    fn render(&mut self, input: BracketRounds<'_, T>);
}

/// An [`Iterator`] over all [`BracketRound`]s within a tournament [`System`].
#[derive(Clone, Debug)]
pub struct BracketRounds<'a, T>
where
    T: System,
{
    tournament: &'a T,
    range: Range<usize>,
}

impl<'a, T> BracketRounds<'a, T>
where
    T: System,
{
    pub(crate) fn new(tournament: &'a T) -> Self {
        Self {
            tournament,
            range: 0..tournament.matches().len(),
        }
    }
}

impl<'a, T> Iterator for BracketRounds<'a, T>
where
    T: System,
{
    type Item = BracketRound<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next bracket round between `self.range`.
        let range = self.tournament.next_bracket_round(self.range.clone());

        log::debug!("Rendering next BracketRound: {:?}", range);

        if range.is_empty() {
            None
        } else {
            // Set the next round to be after the current round (`range`).
            self.range.start = range.end;

            Some(BracketRound::new(self.tournament, range))
        }
    }
}

/// An [`Iterator`] over all [`Bracket`]s in a `BracketRound`.
#[derive(Clone, Debug)]
pub struct BracketRound<'a, T>
where
    T: System,
{
    tournament: &'a T,
    range: Range<usize>,
}

impl<'a, T> BracketRound<'a, T>
where
    T: System,
{
    fn new(tournament: &'a T, range: Range<usize>) -> Self {
        Self { tournament, range }
    }
}

impl<'a, T> Iterator for BracketRound<'a, T>
where
    T: System,
{
    type Item = Bracket<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next bracket between `self.range`.
        let range = self.tournament.next_bracket(self.range.clone());

        log::debug!("Rendering next Bracket: {:?}", range);

        if range.is_empty() {
            None
        } else {
            // Set the next bracket to be after the current bracket (`range`).
            self.range.start = range.end;

            Some(Bracket::new(self.tournament, range))
        }
    }
}

/// An [`Iterator`] over all [`Round`]s in a `Bracket`.
#[derive(Clone, Debug)]
pub struct Bracket<'a, T>
where
    T: System,
{
    tournament: &'a T,
    range: Range<usize>,
}

impl<'a, T> Bracket<'a, T>
where
    T: System,
{
    fn new(tournament: &'a T, range: Range<usize>) -> Self {
        Self { tournament, range }
    }
}

impl<'a, T> Iterator for Bracket<'a, T>
where
    T: System,
{
    type Item = Round<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next round between `self.range`.
        let range = self.tournament.next_round(self.range.clone());

        log::debug!("Rendering next Round: {:?}", range);

        if range.is_empty() {
            None
        } else {
            // Set the next round to be after the current round (`range`).
            self.range.start = range.end;

            Some(Round::new(self.tournament, range))
        }
    }
}

/// An [`Iterator`] over [`Match`]es of a `Round`.
#[derive(Clone, Debug)]
pub struct Round<'a, T>
where
    T: System,
{
    tournament: &'a T,
    start: usize,
    end: usize,
}

impl<'a, T> Round<'a, T>
where
    T: System,
{
    fn new(tournament: &'a T, range: Range<usize>) -> Self {
        Self {
            tournament,
            start: range.start,
            end: range.end,
        }
    }

    pub fn indexed(self) -> Indexed<'a, T> {
        Indexed { iter: self }
    }
}

impl<'a, T> Iterator for Round<'a, T>
where
    T: System,
{
    type Item = (&'a Match<Node<T::NodeData>>, Position);

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            None
        } else {
            let m = &self.tournament.matches()[self.start];
            let position = self.tournament.render_match_position(self.start);

            log::debug!("Rendering next Match {} with {:?}", self.start, position);

            self.start += 1;

            Some((m, position))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Indexed<'a, T>
where
    T: System,
{
    iter: Round<'a, T>,
}

impl<'a, T> Iterator for Indexed<'a, T>
where
    T: System,
{
    type Item = (usize, &'a Match<Node<T::NodeData>>, Position);

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.iter.start;

        self.iter.next().map(|(m, pos)| (index, m, pos))
    }
}

/// The rendering position of a match within a round. The default value is `SpaceAround`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    SpaceAround,
    Bottom(i32),
}

impl Position {
    pub fn bottom(value: i32) -> Self {
        Self::Bottom(value)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::SpaceAround
    }
}
