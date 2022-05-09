//! Bracket Generator
mod double_elimination;
mod single_elimination;
mod utils;

pub use double_elimination::DoubleElimination;
pub use single_elimination::SingleElimination;
use utils::SmallOption;

use thiserror::Error;

use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::result;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Entrants<T> {
    entrants: Vec<T>,
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

impl<T> Deref for Entrants<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.entrants
    }
}

impl<T> DerefMut for Entrants<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entrants
    }
}

impl<T, U> PartialEq<U> for Entrants<T>
where
    T: PartialEq,
    U: AsRef<[T]>,
{
    fn eq(&self, other: &U) -> bool {
        self.entrants == other.as_ref()
    }
}

impl<T> From<Vec<T>> for Entrants<T> {
    fn from(entrants: Vec<T>) -> Self {
        Self { entrants }
    }
}

#[derive(Clone, Debug, Default)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Matches<T> {
    matches: Vec<Match<T>>,
}

impl<T> Matches<T> {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
        }
    }

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
    pub unsafe fn from_raw_parts(ptr: *mut Match<T>, length: usize, capacity: usize) -> Self {
        Self {
            matches: Vec::from_raw_parts(ptr, length, capacity),
        }
    }
}

impl<T> Deref for Matches<T> {
    type Target = Vec<Match<T>>;

    fn deref(&self) -> &Self::Target {
        &self.matches
    }
}

impl<T> DerefMut for Matches<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.matches
    }
}

impl<T, U> PartialEq<U> for Matches<T>
where
    T: PartialEq,
    U: AsRef<[Match<T>]>,
{
    fn eq(&self, other: &U) -> bool {
        self.matches == other.as_ref()
    }
}

impl<T> From<Vec<Match<T>>> for Matches<T> {
    fn from(matches: Vec<Match<T>>) -> Self {
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

/// An entrant in a match.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Entrant<D> {
    pub index: usize,
    #[cfg_attr(feature = "serde-flatten", serde(flatten))]
    pub data: D,
}

impl<D> Entrant<D> {
    pub fn new(index: usize) -> Self
    where
        D: Default,
    {
        Self {
            index,
            data: D::default(),
        }
    }

    pub fn new_with_data(index: usize, data: D) -> Self {
        Self { index, data }
    }

    /// Returns the entrant `T` associated with the current node.
    pub fn entrant<'a, T, U>(&self, entrants: &'a U) -> &'a T
    where
        U: AsRef<Entrants<T>>,
    {
        unsafe { entrants.as_ref().get_unchecked(self.index) }
    }
}

#[derive(Debug)]
pub struct EntrantRefMut<'a, T, D> {
    index: usize,
    entrant: &'a T,
    data: &'a mut D,
}

impl<'a, T, D> EntrantRefMut<'a, T, D> {
    pub(crate) fn new(index: usize, entrant: &'a T, data: &'a mut D) -> Self {
        Self {
            index,
            entrant,
            data,
        }
    }
}

impl<'a, T, D> AsRef<T> for EntrantRefMut<'a, T, D> {
    fn as_ref(&self) -> &T {
        self.entrant
    }
}

impl<'a, T, D> Deref for EntrantRefMut<'a, T, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'a, T, D> DerefMut for EntrantRefMut<'a, T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// An `Result<T>` using [`enum@Error`] as an error type.
pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Debug, Error, PartialEq)]
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

    pub fn winner<'a, T>(
        &mut self,
        entrant: &EntrantSpot<EntrantRefMut<'a, T, D>>,
        data: D,
    ) -> &mut Self {
        self.winner = Some((entrant.as_ref().map(|e| e.index), data));
        self
    }

    pub fn winner_default<'a, T>(
        &mut self,
        entrant: &EntrantSpot<EntrantRefMut<'a, T, D>>,
    ) -> &mut Self
    where
        D: Default,
    {
        self.winner(entrant, D::default())
    }

    pub fn loser<'a, T>(
        &mut self,
        entrant: &EntrantSpot<EntrantRefMut<'a, T, D>>,
        data: D,
    ) -> &mut Self {
        self.loser = Some((entrant.as_ref().map(|e| e.index), data));
        self
    }

    pub fn loser_default<'a, T>(
        &mut self,
        entrant: &EntrantSpot<EntrantRefMut<'a, T, D>>,
    ) -> &mut Self
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
    pub fn new(entrants: [EntrantSpot<T>; 2]) -> Self {
        Self { entrants }
    }

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
    /// Calling this method with an `index` that is out-of-bounds is unidentified behavoir.
    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &EntrantSpot<T> {
        self.entrants.get_unchecked(index)
    }

    /// Returns a mutable reference to the entrant at `index` without checking bounds.
    ///
    /// # Safety
    ///
    /// Calling this method with an `index` that is out-of-bounds is unidentified behavoir.
    #[inline]
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut EntrantSpot<T> {
        self.entrants.get_unchecked_mut(index)
    }
}

impl<T> Index<usize> for Match<T> {
    type Output = EntrantSpot<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entrants[index]
    }
}

impl<T> IndexMut<usize> for Match<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entrants[index]
    }
}

impl<D> Match<Entrant<D>> {
    /// Converts this `Match<Entrant<D>>` into a `Match<EntrantRef<'a, T, D>>` with the referenced
    /// entrant `T` from `entrants`.
    pub(crate) fn to_ref_mut<'a, T>(
        &'a mut self,
        entrants: &'a Entrants<T>,
    ) -> Match<EntrantRefMut<'a, T, D>> {
        let mut array: [MaybeUninit<EntrantSpot<EntrantRefMut<'_, T, D>>>; 2] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (elem, entrant) in array.iter_mut().zip(self.entrants.iter_mut()) {
            match entrant {
                EntrantSpot::Entrant(ref mut e) => {
                    let entrant = unsafe { entrants.get_unchecked(e.index) };

                    elem.write(EntrantSpot::Entrant(EntrantRefMut::new(
                        e.index,
                        entrant,
                        &mut e.data,
                    )));
                }
                EntrantSpot::Empty => {
                    elem.write(EntrantSpot::Empty);
                }
                EntrantSpot::TBD => {
                    elem.write(EntrantSpot::TBD);
                }
            }
        }

        Match {
            // SAFETY: Every element in `array` has been initialized.
            entrants: unsafe { std::mem::transmute(array) },
        }
    }
}

/// An iterator over all rounds of a [`SingleElimination`] tournament.
#[derive(Debug)]
pub struct RoundsIter<'a, T> {
    slice: &'a [Match<T>],
    index: usize,
    /// The number of matches in the next round.
    next_round: usize,
}

impl<'a, T> RoundsIter<'a, T> {
    pub fn with_index(self) -> RoundsIterIndex<'a, T> {
        RoundsIterIndex {
            slice: self.slice,
            index: self.index,
            next_round: self.next_round,
        }
    }
}

impl<'a, T> Iterator for RoundsIter<'a, T> {
    type Item = &'a [Match<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let slice = &self.slice[self.index..self.index + self.next_round];

            self.index += self.next_round;
            self.next_round /= 2;

            Some(slice)
        } else {
            None
        }
    }
}

/// An iterator over all rounds and their starting indexes in the [`SingleElimination`]
/// tournament.
pub struct RoundsIterIndex<'a, T> {
    slice: &'a [Match<T>],
    index: usize,
    /// The number of matches in the next round.
    next_round: usize,
}

impl<'a, T> Iterator for RoundsIterIndex<'a, T> {
    type Item = (&'a [Match<T>], usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let slice = &self.slice[self.index..self.index + self.next_round];
            let index = self.index;

            self.index += self.next_round;
            self.next_round /= 2;

            Some((slice, index))
        } else {
            None
        }
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

    pub fn is_entrant(&self) -> bool {
        matches!(self, Self::Entrant(_))
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

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

    pub fn as_ref(&self) -> EntrantSpot<&T> {
        match *self {
            Self::Entrant(ref entrant) => EntrantSpot::Entrant(entrant),
            Self::Empty => EntrantSpot::Empty,
            Self::TBD => EntrantSpot::TBD,
        }
    }

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
#[derive(Copy, Clone, Debug, PartialEq)]
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
    fn default() -> Self {
        Self::new()
    }
}

impl<S> EntrantData for EntrantScore<S>
where
    S: Default,
{
    fn reset(&mut self) {
        self.score = S::default();
        self.winner = false;
    }

    fn set_winner(&mut self, winner: bool) {
        self.winner = winner;
    }
}

impl<T> From<T> for EntrantSpot<T>
where
    T: EntrantData,
{
    fn from(entrant: T) -> Self {
        Self::Entrant(entrant)
    }
}

#[derive(Debug)]
pub struct LowerBracketIter<'a, T> {
    slice: &'a [Match<T>],
    start: usize,
    index: usize,
    num_matches: usize,
    iter_count: u8,
}

impl<'a, T> LowerBracketIter<'a, T> {
    pub fn with_index(self) -> LowerBracketIndexIter<'a, T> {
        LowerBracketIndexIter {
            start: self.start,
            slice: self.slice,
            index: self.index,
            num_matches: self.num_matches,
            iter_count: self.iter_count,
        }
    }
}

impl<'a, T> Iterator for LowerBracketIter<'a, T> {
    type Item = &'a [Match<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.start + 1 < self.slice.len() {
            let slice =
                &self.slice[self.start + self.index..self.start + self.index + self.num_matches];

            self.index += self.num_matches;
            self.num_matches = match self.iter_count {
                0 => {
                    self.iter_count += 1;
                    self.num_matches
                }
                _ => {
                    self.iter_count = 0;
                    self.num_matches / 2
                }
            };

            Some(slice)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct LowerBracketIndexIter<'a, T> {
    slice: &'a [Match<T>],
    start: usize,
    index: usize,
    num_matches: usize,
    iter_count: u8,
}

impl<'a, T> Iterator for LowerBracketIndexIter<'a, T> {
    type Item = (&'a [Match<T>], usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.start + 1 < self.slice.len() {
            let slice =
                &self.slice[self.start + self.index..self.start + self.index + self.num_matches];

            let index = self.index + self.start;

            self.index += self.num_matches;
            self.num_matches = match self.iter_count {
                0 => {
                    self.iter_count += 1;
                    self.num_matches
                }
                _ => {
                    self.iter_count = 0;
                    self.num_matches / 2
                }
            };

            Some((slice, index))
        } else {
            None
        }
    }
}

pub struct FinalBracketIter<'a, T> {
    slice: &'a [Match<T>],
    start: usize,
    index: usize,
}

impl<'a, T> FinalBracketIter<'a, T> {
    pub fn with_index(self) -> FinalBracketIndexIter<'a, T> {
        FinalBracketIndexIter {
            slice: self.slice,
            start: self.start,
            index: self.index,
        }
    }
}

impl<'a, T> Iterator for FinalBracketIter<'a, T> {
    type Item = &'a Match<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.start < self.slice.len() {
            let slice = &self.slice[self.start + self.index];

            self.index += 1;

            Some(slice)
        } else {
            None
        }
    }
}

pub struct FinalBracketIndexIter<'a, T> {
    slice: &'a [Match<T>],
    start: usize,
    index: usize,
}

impl<'a, T> Iterator for FinalBracketIndexIter<'a, T> {
    type Item = (&'a Match<T>, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + self.start < self.slice.len() {
            let slice = &self.slice[self.start + self.index];
            let index = self.start + self.index;

            self.index += 1;

            Some((slice, index))
        } else {
            None
        }
    }
}

/// Information about the next match.
///
/// # Safety
///
/// The methods on `NextMatches` assume that the given indexes and positions are valid, as long as
/// those values are not `None`.
///
/// Calling methods with an value that is out-of-bounds for the given matches is unidentified
/// behavoir.
#[derive(Clone, Debug)]
pub struct NextMatches {
    winner_index: SmallOption<usize>,
    pub(crate) winner_position: usize,
    loser_index: SmallOption<usize>,
    pub(crate) loser_position: usize,
}

impl NextMatches {
    pub fn winner_match_mut<'a, T>(&self, matches: &'a mut Matches<T>) -> Option<&'a mut Match<T>> {
        if self.winner_index.is_some() {
            unsafe { Some(matches.get_unchecked_mut(*self.winner_index)) }
        } else {
            None
        }
    }

    pub fn loser_match_mut<'a, T>(&self, matches: &'a mut Matches<T>) -> Option<&'a mut Match<T>> {
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

    pub fn winner_mut<'a, T>(&self, matches: &'a mut Matches<T>) -> Option<&'a mut EntrantSpot<T>> {
        if self.winner_index.is_some() {
            unsafe {
                let r#match = matches.get_unchecked_mut(*self.winner_index);

                Some(r#match.get_unchecked_mut(self.winner_position))
            }
        } else {
            None
        }
    }

    pub fn loser_mut<'a, T>(&self, matches: &'a mut Matches<T>) -> Option<&'a mut EntrantSpot<T>> {
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
    fn default() -> Self {
        Self {
            winner_index: SmallOption::none(),
            winner_position: 0,
            loser_index: SmallOption::none(),
            loser_position: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EntrantData;

    #[macro_export]
    macro_rules! entrants {
        ($($x:expr),*) => {
            vec![$($x),*].into_iter()
        };
    }

    impl EntrantData for u32 {
        fn set_winner(&mut self, _winner: bool) {}
        fn reset(&mut self) {}
    }
}
