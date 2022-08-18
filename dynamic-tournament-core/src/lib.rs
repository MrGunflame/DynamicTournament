//! # dynamic-tournament-core
//!
//! This crate contains all the items required to build tournament brackets. It also currently
//! contains two builtin tournaments: [`SingleElimination`] and [`DoubleElimination`].
//!
//! Important types:
//! - [`System`]: A trait used to describe a tournament. This should be implemented on any
//! tournament type.
//! - [`Entrants`]: A wrapper around `Vec<T>` where `T` is an entrant in a tournament.
//! - [`Matches`]: A `Vec` of matches contained in the tournament.
//! - [`Match`]: A *match* or *heat* of two parties.
//! - [`EntrantSpot`]: A *spot* within a match, which can contain an entrant, be permanently empty
//! or contain a to-be-done spot.
//! - [`Node`]: The data contained in every match. Includes a reference to the entrant.
//! - [`EntrantScore`]: A score and a winner flag. Can be used together with any integer.
//!
//! ## Feature Flags
//!
//! `serde`: Adds `Serialize` and `Deserialize` impls to almost all types.
//!
pub mod options;
pub mod render;

mod double_elimination;
mod single_elimination;
pub mod tournament;
mod utils;

pub use double_elimination::DoubleElimination;
use render::{BracketRounds, Position, Renderer};
pub use single_elimination::SingleElimination;
use utils::SmallOption;

use thiserror::Error;

use std::borrow::Borrow;
use std::ops::{Deref, DerefMut, Index, IndexMut, Range};
use std::result;
use std::vec::IntoIter;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A wrapper around a `Vec<T>` where `T` should be considered an entrant for a tournament.
///
/// This is a wrapper around a `Vec<T>` and has the same layout as a `Vec<T>`.
#[derive(Clone, Debug, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Entrants<T> {
    entrants: Vec<T>,
}

impl<T> Entrants<T> {
    /// Creates a new empty `Entrants` list.
    #[inline]
    pub fn new() -> Self {
        Self {
            entrants: Vec::new(),
        }
    }

    /// Creates a new empty `Entrants` list with the specified capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entrants: Vec::with_capacity(capacity),
        }
    }
}

impl<T> FromIterator<T> for Entrants<T> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let entrants = iter.into_iter().collect();

        Self { entrants }
    }
}

impl<T> IntoIterator for Entrants<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.entrants.into_iter()
    }
}

impl<T> Deref for Entrants<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entrants
    }
}

impl<T> DerefMut for Entrants<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entrants
    }
}

impl<T, U> PartialEq<U> for Entrants<T>
where
    T: PartialEq,
    U: AsRef<[T]>,
{
    #[inline]
    fn eq(&self, other: &U) -> bool {
        self.entrants == other.as_ref()
    }
}

impl<T> From<Vec<T>> for Entrants<T> {
    #[inline]
    fn from(entrants: Vec<T>) -> Self {
        Self { entrants }
    }
}

/// A wrapper around a `Vec<Match<Node<T>>>` where `T` should be considered a [`EntrantData`] value
/// stored which is stored in each [`Node`].
///
/// This is a wrapper around a `Vec<Match<Node<T>>>` and has the same layout as a
/// `Vec<Match<Node<T>>>`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Matches<T> {
    matches: Vec<Match<Node<T>>>,
}

impl<T> Matches<T> {
    #[inline]
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            matches: Vec::with_capacity(capacity),
        }
    }

    /// Creates a `Matches<T>` from its raw parts.
    ///
    /// # Safety
    ///
    /// See [`Vec::from_raw_parts`]
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut Match<Node<T>>, length: usize, capacity: usize) -> Self {
        Self {
            matches: Vec::from_raw_parts(ptr, length, capacity),
        }
    }
}

impl<T> Deref for Matches<T> {
    type Target = Vec<Match<Node<T>>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.matches
    }
}

impl<T> DerefMut for Matches<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matches
    }
}

impl<T, U> PartialEq<U> for Matches<T>
where
    T: PartialEq,
    U: AsRef<[Match<Node<T>>]>,
{
    #[inline]
    fn eq(&self, other: &U) -> bool {
        self.matches == other.as_ref()
    }
}

impl<T> From<Vec<Match<Node<T>>>> for Matches<T> {
    #[inline]
    fn from(matches: Vec<Match<Node<T>>>) -> Self {
        Self { matches }
    }
}

/// Some data that is stored within the bracket of the tournament. This is usually a score or
/// something similar. See [`EntrantScore`] for an example.
pub trait EntrantData: Default {
    /// Sets the winner state of the data to `winner`.
    fn set_winner(&mut self, winner: bool);
    /// Resets the data. This should cause the `Self` become the same value as `Self::default()`.
    fn reset(&mut self);
}

/// A data value which is stored for each spot in a match that contains an entrant.
///
/// Since `Node` is stored a lot of times for a single tournament `D` should either implement
/// [`Copy`] or should be cheaply clonable. `D` should also never contain any data directly related
/// to the entrant (like the entrants name).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Node<D> {
    index: usize,
    #[cfg_attr(feature = "serde-flatten", serde(flatten))]
    pub data: D,
}

impl<D> Node<D> {
    /// Creates a new `Node` using the given `index` and the default value for `D`.
    #[inline]
    pub fn new(index: usize) -> Self
    where
        D: Default,
    {
        Self {
            index,
            data: D::default(),
        }
    }

    #[inline]
    pub fn new_with_data(index: usize, data: D) -> Self {
        Self { index, data }
    }

    /// Returns the entrant `T` associated with the current node.
    #[inline]
    pub fn entrant<'a, T, U>(&self, entrants: &'a U) -> Option<&'a T>
    where
        U: Borrow<Entrants<T>>,
    {
        entrants.borrow().get(self.index)
    }

    /// Returns the entrant `T` associated with the current node without checking the bounds of
    /// `entrants`.
    ///
    /// This method is useful and safe to use if you are certain that the [`Entrants`] come from
    /// the same tournament as the `Node`.
    ///
    /// # Safety
    ///
    /// Calling this method with an [`Entrants`] value with a length equal to or smaller than the
    /// index stored in the `Node` causes undefined behavoir.
    #[inline]
    pub unsafe fn entrant_unchecked<'a, T, U>(&self, entrants: &'a U) -> &'a T
    where
        U: Borrow<Entrants<T>>,
    {
        entrants.borrow().get_unchecked(self.index)
    }
}

/// An `Result<T>` using [`enum@Error`] as an error type.
pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("invalid number of matches: expected {expected}, found {found}")]
    InvalidNumberOfMatches { expected: usize, found: usize },
    #[error(
        "invalid entrant: match refers to entrant at {index} but only {length} entrants are given"
    )]
    InvalidEntrant { index: usize, length: usize },
}

/// The result of a [`Match`].
#[derive(Clone, Debug, Default)]
pub struct MatchResult<D> {
    pub(crate) winner: Option<(EntrantSpot<usize>, D)>,
    pub(crate) loser: Option<(EntrantSpot<usize>, D)>,
    pub(crate) reset: bool,
}

impl<D> MatchResult<D> {
    /// Creates a new `MatchResult` with the default state.
    #[inline]
    pub fn new() -> Self {
        Self {
            winner: None,
            loser: None,
            reset: false,
        }
    }

    /// Resets the match the default state of `D`.
    pub fn reset_default(&mut self) -> &mut Self
    where
        D: Default,
    {
        self.winner = Some((EntrantSpot::TBD, D::default()));
        self.loser = Some((EntrantSpot::TBD, D::default()));
        self.reset = true;

        self
    }

    pub fn winner(&mut self, entrant: &EntrantSpot<Node<D>>, data: D) -> &mut Self {
        self.winner = Some((entrant.as_ref().map(|e| e.index), data));
        self
    }

    /// Sets the winner of this [`Match`] and uses `D::default` as the value for the next [`Match`].
    #[inline]
    pub fn winner_default(&mut self, entrant: &EntrantSpot<Node<D>>) -> &mut Self
    where
        D: Default,
    {
        self.winner(entrant, D::default())
    }

    pub fn loser(&mut self, entrant: &EntrantSpot<Node<D>>, data: D) -> &mut Self {
        self.loser = Some((entrant.as_ref().map(|e| e.index), data));
        self
    }

    /// Sets the loser of this [`Match`] and uses `D::default` as the value for the next [`Match`].
    #[inline]
    pub fn loser_default(&mut self, entrant: &EntrantSpot<Node<D>>) -> &mut Self
    where
        D: Default,
    {
        self.loser(entrant, D::default())
    }
}

/// A match consisting of at 2 parties.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Match<T> {
    pub entrants: [EntrantSpot<T>; 2],
}

impl<T> Match<T> {
    #[inline]
    pub fn new(entrants: [EntrantSpot<T>; 2]) -> Self {
        Self { entrants }
    }

    #[inline]
    pub(crate) fn is_placeholder(&self) -> bool {
        matches!(self.entrants[0], EntrantSpot::Empty)
            || matches!(self.entrants[1], EntrantSpot::Empty)
    }

    /// Returns a reference to the entrant at `index`.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&EntrantSpot<T>> {
        self.entrants.get(index)
    }

    /// Returns a mutable reference to the entrant at `index`.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut EntrantSpot<T>> {
        self.entrants.get_mut(index)
    }

    /// Returns a reference to the entrant at `index` without checking bounds.
    ///
    /// # Safety
    ///
    /// Calling this method with an `index` that is out-of-bounds is undefined behavoir.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &EntrantSpot<T> {
        self.entrants.get_unchecked(index)
    }

    /// Returns a mutable reference to the entrant at `index` without checking bounds.
    ///
    /// # Safety
    ///
    /// Calling this method with an `index` that is out-of-bounds is undefined behavoir.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut EntrantSpot<T> {
        self.entrants.get_unchecked_mut(index)
    }
}

impl<T> Index<usize> for Match<T> {
    type Output = EntrantSpot<T>;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.entrants[index]
    }
}

impl<T> IndexMut<usize> for Match<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entrants[index]
    }
}

/// A spot for an Entrant in the bracket.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum EntrantSpot<T> {
    Entrant(T),
    Empty,
    TBD,
}

impl<T> EntrantSpot<T> {
    /// Creates a new `EntrantSpot` from an [`Option`]. A `Some(T)` value will translate into
    /// a `Entrant(T)` value, a `None` value will translate into a `Empty` value.
    pub fn new(entrant: Option<T>) -> Self {
        match entrant {
            Some(entrant) => Self::Entrant(entrant),
            None => Self::Empty,
        }
    }

    /// Returns `true` if the `EntrantSpot` is [`Entrant`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::EntrantSpot;
    /// let spot = EntrantSpot::Entrant(());
    /// assert!(spot.is_entrant());
    /// ```
    /// [`Entrant`]: Self::Entrant
    pub fn is_entrant(&self) -> bool {
        matches!(self, Self::Entrant(_))
    }

    /// Returns `true` if the `EntrantSpot` is [`Empty`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::EntrantSpot;
    /// let spot: EntrantSpot<()> = EntrantSpot::Empty;
    /// assert!(spot.is_empty());
    /// ```
    ///
    /// [`Empty`]: Self::Empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Returns `true` if the `EntrantSpot` is [`TBD`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::EntrantSpot;
    /// let spot: EntrantSpot<()> = EntrantSpot::TBD;
    /// assert!(spot.is_tbd());
    /// ```
    ///
    /// [`TBD`]: Self::TBD
    pub fn is_tbd(&self) -> bool {
        matches!(self, Self::TBD)
    }

    /// Takes out an the value, leaving [`Self::Empty`] in its place.
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Empty)
    }

    /// Unwraps the `self` value, panicking if it is not [`Self::Entrant`].
    ///
    /// # Panics
    ///
    /// This method panics when `self` is not [`Self::Entrant`].
    pub fn unwrap(self) -> T {
        match self {
            Self::Entrant(entrant) => entrant,
            _ => panic!(
                "called unwrap on a value of EntrantSpot::{}",
                match self {
                    Self::Empty => "Empty",
                    Self::TBD => "TBD",
                    _ => unreachable!(),
                }
            ),
        }
    }

    /// Unwraps the `self` value, panicking if it is not [`Self::Entrant`].
    ///
    /// # Panics
    ///
    /// This method panics when `self` is not [`Self::Entrant`].
    pub fn unwrap_ref(&self) -> &T {
        match self {
            Self::Entrant(entrant) => entrant,
            _ => panic!(
                "called unwrap on a value of EntrantSpot::{}",
                match self {
                    Self::Empty => "Empty",
                    Self::TBD => "TBD",
                    _ => unreachable!(),
                }
            ),
        }
    }

    /// Unwraps the `self` value, panicking if it is not [`Self::Entrant`].
    ///
    /// # Panics
    ///
    /// This method panics when `self` is not [`Self::Entrant`].
    pub fn unwrap_ref_mut(&mut self) -> &mut T {
        match self {
            Self::Entrant(ref mut entrant) => entrant,
            _ => panic!(
                "called unwrap on a value of EntrantSpot::{}",
                match self {
                    Self::Empty => "Empty",
                    Self::TBD => "TBD",
                    _ => unreachable!(),
                }
            ),
        }
    }

    /// Converts an `&EntrantSpot<T>` into an `EntrantSpot<&T>`.
    pub fn as_ref(&self) -> EntrantSpot<&T> {
        match *self {
            Self::Entrant(ref entrant) => EntrantSpot::Entrant(entrant),
            Self::Empty => EntrantSpot::Empty,
            Self::TBD => EntrantSpot::TBD,
        }
    }

    /// Converts an `&mut EntrantSpot<T>` into an `EntrantSpot<&mut T>`.
    pub fn as_mut(&mut self) -> EntrantSpot<&mut T> {
        match *self {
            Self::Entrant(ref mut entrant) => EntrantSpot::Entrant(entrant),
            Self::Empty => EntrantSpot::Empty,
            Self::TBD => EntrantSpot::TBD,
        }
    }

    /// Maps `EntrantSpot<T>` to `EntrantSpot<U>` by applying `f` on it.
    pub fn map<U, F>(self, f: F) -> EntrantSpot<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Entrant(entrant) => EntrantSpot::Entrant(f(entrant)),
            Self::Empty => EntrantSpot::Empty,
            Self::TBD => EntrantSpot::TBD,
        }
    }
}

/// A score `S` and a winner state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EntrantScore<S> {
    pub score: S,
    pub winner: bool,
}

impl<S> EntrantScore<S>
where
    S: Default,
{
    /// Creates a new `EntrantWithScore` with a score of 0.
    #[inline]
    pub fn new() -> Self {
        EntrantScore {
            score: S::default(),
            winner: false,
        }
    }
}

impl<S> Default for EntrantScore<S>
where
    S: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<S> EntrantData for EntrantScore<S>
where
    S: Default,
{
    #[inline]
    fn reset(&mut self) {
        self.score = S::default();
        self.winner = false;
    }

    #[inline]
    fn set_winner(&mut self, winner: bool) {
        self.winner = winner;
    }
}

impl<T> From<T> for EntrantSpot<T>
where
    T: EntrantData,
{
    #[inline]
    fn from(entrant: T) -> Self {
        Self::Entrant(entrant)
    }
}

/// Information about the next match.
///
/// # Safety
///
/// The methods on `NextMatches` assume that the given indexes and positions are valid, as long as
/// those values are not `None`.
///
/// Calling methods with an value that is out-of-bounds for the given matches is undefined
/// behavoir.
#[derive(Clone, Debug)]
pub struct NextMatches {
    winner_index: SmallOption<usize>,
    pub(crate) winner_position: usize,
    loser_index: SmallOption<usize>,
    pub(crate) loser_position: usize,
}

impl NextMatches {
    pub fn winner_match_mut<'a, T>(
        &self,
        matches: &'a mut Matches<T>,
    ) -> Option<&'a mut Match<Node<T>>> {
        if self.winner_index.is_some() {
            unsafe { Some(matches.get_unchecked_mut(*self.winner_index)) }
        } else {
            None
        }
    }

    pub fn loser_match_mut<'a, T>(
        &self,
        matches: &'a mut Matches<T>,
    ) -> Option<&'a mut Match<Node<T>>> {
        if self.loser_index.is_some() {
            unsafe { Some(matches.get_unchecked_mut(*self.loser_index)) }
        } else {
            None
        }
    }

    pub fn new(winner: Option<(usize, usize)>, loser: Option<(usize, usize)>) -> Self {
        let (winner_index, winner_position) = winner
            .map(|(index, position)| (SmallOption::new_unchecked(index), position))
            .unwrap_or_default();

        let (loser_index, loser_position) = loser
            .map(|(index, position)| (SmallOption::new_unchecked(index), position))
            .unwrap_or_default();

        Self {
            winner_index,
            winner_position,
            loser_index,
            loser_position,
        }
    }

    pub fn winner_mut<'a, T>(
        &self,
        matches: &'a mut Matches<T>,
    ) -> Option<&'a mut EntrantSpot<Node<T>>> {
        if self.winner_index.is_some() {
            unsafe {
                let r#match = matches.get_unchecked_mut(*self.winner_index);

                Some(r#match.get_unchecked_mut(self.winner_position))
            }
        } else {
            None
        }
    }

    pub fn loser_mut<'a, T>(
        &self,
        matches: &'a mut Matches<T>,
    ) -> Option<&'a mut EntrantSpot<Node<T>>> {
        if self.loser_index.is_some() {
            unsafe {
                let r#match = matches.get_unchecked_mut(*self.loser_index);

                Some(r#match.get_unchecked_mut(self.loser_position))
            }
        } else {
            None
        }
    }
}

impl Default for NextMatches {
    #[inline]
    fn default() -> Self {
        Self {
            winner_index: SmallOption::none(),
            winner_position: 0,
            loser_index: SmallOption::none(),
            loser_position: 0,
        }
    }
}

/// A tournament system.
pub trait System: Sized + Borrow<Entrants<Self::Entrant>> {
    type Entrant;
    type NodeData: EntrantData;

    /// Returns a reference to the [`Entrants`] of the `Tournament`.
    fn entrants(&self) -> &Entrants<Self::Entrant>;

    /// Returns a mutable reference to the [`Entrants`] of the `Tournament`.
    ///
    /// # Safety
    ///
    /// Removing elements from [`Entrants`] while there are still [`Node`]s with an `index`
    /// pointing to that element in the `Tournament` is undefined behavoir.
    ///
    /// Growing [`Entrants`] or modifying elements is always safe.
    unsafe fn entrants_mut(&mut self) -> &mut Entrants<Self::Entrant>;

    /// Consumes the `Tournament`, returning the [`Entrants`] of the `Tournament`.
    fn into_entrants(self) -> Entrants<Self::Entrant>;

    /// Returns a reference to the [`Matches`] of the `Tournament`.
    fn matches(&self) -> &Matches<Self::NodeData>;

    /// Returns a mutable reference to the [`Matches`] of the `Tournament`.
    ///
    /// # Safety
    ///
    /// Changing the length of the [`Matches`] to a length that is invalid for the `Tournament`
    /// is undefined behavoir. The exact requirements depend on the concrete `Tournament`. Changing
    /// the `index` field [`Node`]s to an out-of-bounds of [`Entrants`] is undefined behavoir.
    ///
    /// Changing the `data` fields of [`Node`]s is always safe, but may cause the `Tournament`
    /// to be in an incorrect or inconsistent state.
    unsafe fn matches_mut(&mut self) -> &mut Matches<Self::NodeData>;

    /// Consumes the `Tournament`, returning the [`Matches`] of the `Tournament`.
    fn into_matches(self) -> Matches<Self::NodeData>;

    /// Returns the [`NextMatches`] of the match with the given `index`.
    fn next_matches(&self, index: usize) -> NextMatches;

    /// Updates the match at `index` by applying `f` on it. The next match is updated using the
    /// returned [`MatchResult`]. If `index` is out-of-bounds, `f` is never called.
    fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<Self::NodeData>>, &mut MatchResult<Self::NodeData>);

    /// Returns the next bracket round between `range`. If `range` is empty or no bracket rounds
    /// are between `range`, `0..0` should be returned.
    fn next_bracket_round(&self, range: Range<usize>) -> Range<usize>;

    /// Returns the next bracket between `range`.
    fn next_bracket(&self, range: Range<usize>) -> Range<usize>;

    /// Returns the next round between `range`.
    fn next_round(&self, range: Range<usize>) -> Range<usize>;

    fn render_match_position(&self, _index: usize) -> Position {
        Position::default()
    }

    /// Renders the tournament using the given [`Renderer`].
    fn render<R>(&self, renderer: &mut R)
    where
        R: Renderer<Self, Self::Entrant, Self::NodeData>,
    {
        renderer.render(BracketRounds::new(self));
    }
}

#[cfg(test)]
mod tests {
    use crate::{render::Renderer, EntrantSpot};

    use super::{BracketRounds, EntrantData, Match, Node, System};

    #[macro_export]
    macro_rules! entrants {
        ($($x:expr),*) => {
            vec![$($x),*].into_iter()
        };
    }

    #[macro_export]
    macro_rules! option_values {
        ($($key:expr => $val:expr),*$(,)?) => {{
            let mut options = $crate::options::TournamentOptionValues::new();
            $(
                options.set($key, $val);
            )*

            options
        }};
    }

    impl EntrantData for u32 {
        fn set_winner(&mut self, _winner: bool) {}
        fn reset(&mut self) {}
    }

    #[derive(Debug, Default)]
    pub struct TestRenderer {
        #[allow(clippy::type_complexity)]
        matches: Vec<Vec<Vec<Vec<Match<Node<u32>>>>>>,
    }

    impl<T, E, D> Renderer<T, E, D> for TestRenderer
    where
        T: System<Entrant = E, NodeData = D>,
    {
        fn render(&mut self, input: BracketRounds<'_, T>) {
            for bracket_round in input {
                let mut brackets = Vec::new();

                for bracket in bracket_round {
                    let mut rounds = Vec::new();

                    for round in bracket {
                        let mut matches = Vec::new();

                        for r#match in round {
                            let mut indexes = [EntrantSpot::Empty, EntrantSpot::Empty];

                            for (index, entrant) in r#match.0.entrants.iter().enumerate() {
                                indexes[index] =
                                    entrant.as_ref().map(|entrant| Node::new(entrant.index));
                            }

                            matches.push(Match::new(indexes));
                        }

                        rounds.push(matches);
                    }

                    brackets.push(rounds);
                }

                self.matches.push(brackets);
            }
        }
    }

    impl PartialEq<Vec<Vec<Vec<Vec<Match<Node<u32>>>>>>> for TestRenderer {
        fn eq(&self, other: &Vec<Vec<Vec<Vec<Match<Node<u32>>>>>>) -> bool {
            &self.matches == other
        }
    }
}
