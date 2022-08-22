use crate::options::{OptionValue, TournamentOptionValues, TournamentOptions};
use crate::render::Position;
use crate::{EntrantData, Entrants, Match, Matches, NextMatches, System};
use crate::{EntrantSpot, Error, MatchResult, Node, Result};

use std::borrow::Borrow;
use std::ops::Range;
use std::ptr;

/// A single elimination tournament.
#[derive(Clone, Debug)]
pub struct SingleElimination<T, D> {
    entrants: Entrants<T>,
    matches: Matches<D>,
    options: SingleEliminationOptions,
}

impl<T, D> SingleElimination<T, D>
where
    D: EntrantData + Default,
{
    /// Creates a new `SingleElimination` tournament with the given `entrants`.
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        Self::new_with_options(entrants, Self::options())
    }

    /// Creates a new `SingleElimination` tournament with the given `entrants` and using the
    /// given `options`.
    ///
    /// If you don't need to specify the options consider using [`new`].
    ///
    /// [`new`]: Self::new
    pub fn new_with_options<I, O>(entrants: I, options: O) -> Self
    where
        I: Iterator<Item = T>,
        O: Into<TournamentOptionValues>,
    {
        let options = SingleEliminationOptions::new(options.into());
        log::debug!("Using options: {:?}", options);

        let entrants: Entrants<T> = entrants.collect();

        log::debug!(
            "Creating new SingleElimination bracket with {} entrants",
            entrants.len()
        );

        let initial_matches = match entrants.len() {
            1 | 2 => 1,
            n => n.next_power_of_two() / 2,
        };

        let mut num_matches = (initial_matches * 2).saturating_sub(1);

        // At least 3 entrants are required for a third place match.
        if entrants.len() > 2 && options.third_place_match {
            num_matches += 1;
        }

        let mut matches = Matches::with_capacity(num_matches);

        // Push the first half entrants into matches. This already creates the minimum number of
        // matches required.
        let mut ptr = matches.as_mut_ptr();
        for index in 0..initial_matches {
            let first = EntrantSpot::Entrant(Node::new(index));
            let second = EntrantSpot::Empty;

            // SAFETY: `matches` has allocated enough memory for at least `initial_matches` items.
            unsafe {
                ptr::write(ptr, Match::new([first, second]));
                ptr = ptr.add(1);
            }
        }

        // SAFETY: The first `initial_matches` items in the buffer has been written to.
        unsafe {
            matches.set_len(initial_matches);
        }

        // Fill the second spots in the matches.
        let mut index = initial_matches;
        while index < entrants.len() {
            // SAFETY: The matches have already been written to the buffer in the first iteration.
            let spot = unsafe {
                matches
                    .get_unchecked_mut(index - initial_matches)
                    .get_unchecked_mut(1)
            };

            *spot = EntrantSpot::Entrant(Node::new(index));
            index += 1;
        }

        // Fill `matches` with `TBD` matches.
        while matches.len() < matches.capacity() {
            matches.push(Match::new([EntrantSpot::TBD, EntrantSpot::TBD]));
        }

        // Forward all placeholder matches.
        // Note: This `if` is necessary because `0.next_power_of_two()` returns 1. This is
        // incorrect when `matches.len() == 0`.
        if entrants.len() > 0 {
            while index < entrants.len().next_power_of_two() {
                let new_index = initial_matches + (index - initial_matches) / 2;
                // SAFETY: `new_index` is in bounds of `matches`, `index % 2` never exceeds 1.
                let spot = unsafe {
                    matches
                        .get_unchecked_mut(new_index)
                        .get_unchecked_mut(index % 2)
                };

                *spot = EntrantSpot::Entrant(Node::new(index - initial_matches));

                index += 1;
            }
        }

        log::debug!(
            "Created new SingleElimination bracket with {} matches",
            matches.len()
        );

        Self {
            entrants,
            matches,
            options,
        }
    }

    /// Returns the [`TournamentOptions`] accepted by this system.
    pub fn options() -> TournamentOptions {
        TournamentOptions::builder()
            .option(
                "third_place_match",
                "Include a match for the third place",
                false,
            )
            .build()
    }

    /// Resumes the bracket from existing matches.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if `matches` has an invalid number of matches for `entrants` or an
    /// [`Node`] in `matches` pointed to a value that is out-of-bounds.
    pub fn resume<O>(entrants: Entrants<T>, matches: Matches<D>, options: O) -> Result<Self>
    where
        O: Into<TournamentOptionValues>,
    {
        let options = options.into();
        log::debug!(
            "Trying to resume SingleElimination bracket with {} entrants and {} matches",
            entrants.len(),
            matches.len()
        );

        let mut expected = Self::calculate_matches(entrants.len());

        // Add third_place_match is set in options.
        if let Some(OptionValue::Bool(v)) = options.get("third_place_match") {
            if *v && entrants.len() > 2 {
                expected += 1;
            }
        }

        let found = matches.len();

        if found != expected {
            return Err(Error::InvalidNumberOfMatches { expected, found });
        }

        for m in matches.iter() {
            for entrant in m.entrants.iter() {
                if let EntrantSpot::Entrant(entrant) = entrant {
                    if entrant.index >= entrants.len() {
                        return Err(Error::InvalidEntrant {
                            index: entrant.index,
                            length: entrants.len(),
                        });
                    }
                }
            }
        }

        // SAFETY: `matches` has a valid length for `entrants` and all indexes are within bounds.
        unsafe { Ok(Self::resume_unchecked(entrants, matches, options)) }
    }

    /// Resumes the bracket from existing matches without validating the length of `matches`.
    ///
    /// # Safety
    ///
    /// Calling this function with a number of `matches` that is not valid for the length of
    /// `entrants` will create an [`SingleElimination`] object with false assumptions. Usage
    /// of that invalid object can cause all sorts behavoir including infinite loops, wrong
    /// returned data and potentially undefined behavoir.
    #[inline]
    pub unsafe fn resume_unchecked<O>(
        entrants: Entrants<T>,
        matches: Matches<D>,
        options: O,
    ) -> Self
    where
        O: Into<TournamentOptionValues>,
    {
        let options = SingleEliminationOptions::new(options.into());
        log::debug!("Using options: {:?}", options);

        log::debug!(
            "Resuming SingleElimination bracket with {} entrants and {} matches",
            entrants.len(),
            matches.len()
        );

        Self {
            entrants,
            matches,
            options,
        }
    }

    /// Returns a reference to the entrants in the tournament.
    #[inline]
    pub fn entrants(&self) -> &Entrants<T> {
        &self.entrants
    }

    /// Returns a mutable reference to the entrants in the tournament.
    ///
    /// # Safety
    ///
    /// [`SingleElimination`] generally assumes that `entrants` has a correct length and capacity
    /// compared to `matches`. Changing the length or capacity of the entrants may cause
    /// undefined behavoir if the new entrants have an incorrect length or capacity compared to
    /// the matches.
    ///
    /// Changing the `entrants` without resizing [`Entrants`] can never cause undefined behavoir.
    #[inline]
    pub unsafe fn entrants_mut(&mut self) -> &mut Entrants<T> {
        &mut self.entrants
    }

    /// Returns the entrants from the tournament.
    #[inline]
    pub fn into_entrants(self) -> Entrants<T> {
        self.entrants
    }

    /// Returns a reference to the matches in the tournament.
    #[inline]
    pub fn matches(&self) -> &Matches<D> {
        &self.matches
    }

    /// Returns a mutable reference to the matches in the tournament.
    ///
    /// # Safety
    ///
    /// [`SingleElimination`] assumes that `matches` has a length of pow(2, n). Violating this
    /// assumption may cause undefined behavoir. Further changing the index field of [`Node`]
    /// to a value that is not in bounds of `entrants` causes undefined behavoir.
    ///
    /// Changing the data field of [`Node`] without changing the length of [`Matches`] or
    /// changing the index field of [`Node`] is always safe, **but may cause the tournament to
    /// be in an incorrect or inconsistent state**.
    #[inline]
    pub unsafe fn matches_mut(&mut self) -> &mut Matches<D> {
        &mut self.matches
    }

    /// Returns the matches from the tournament.
    #[inline]
    pub fn into_matches(self) -> Matches<D> {
        self.matches
    }

    /// Calculates the number of matches required to build a [`SingleElimination`] tournament
    /// using `entrants`-number of entrants.
    fn calculate_matches(entrants: usize) -> usize {
        match entrants {
            1 | 2 => 1,
            n => n.next_power_of_two() - 1,
        }
    }
}

impl<T, D> System for SingleElimination<T, D>
where
    D: EntrantData + Default,
{
    type Entrant = T;
    type NodeData = D;

    #[inline]
    fn entrants(&self) -> &Entrants<Self::Entrant> {
        &self.entrants
    }

    #[inline]
    unsafe fn entrants_mut(&mut self) -> &mut Entrants<T> {
        &mut self.entrants
    }

    #[inline]
    fn into_entrants(self) -> Entrants<T> {
        self.entrants
    }

    #[inline]
    fn matches(&self) -> &Matches<Self::NodeData> {
        &self.matches
    }

    #[inline]
    unsafe fn matches_mut(&mut self) -> &mut Matches<D> {
        &mut self.matches
    }

    #[inline]
    fn into_matches(self) -> Matches<D> {
        self.matches
    }

    fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<D>>, &mut MatchResult<D>),
    {
        // Get the match at `index` or abort.
        // Note: This will borrow `self.matches` mutably until the end of the scope. All
        // operations that access `self.matches` at an index that is **not `index`** are still
        // safe.

        let r#match = match self.matches.get_mut(index) {
            Some(r#match) => r#match,
            None => return,
        };

        let mut res = MatchResult::default();

        f(r#match, &mut res);

        let next_matches = self.next_matches(index);

        log::debug!(
            "Got match results: winner: {:?}, loser: {:?}",
            res.winner.as_ref().map(|(e, _)| e),
            res.loser.as_ref().map(|(e, _)| e),
        );

        if let Some((entrant, data)) = res.winner {
            // Only update the next match if it actually exists.
            if let Some(spot) = next_matches.winner_mut(&mut self.matches) {
                log::debug!("Next winner match is {}", *next_matches.winner_index);

                *spot = match entrant {
                    EntrantSpot::Entrant(index) => {
                        EntrantSpot::Entrant(Node::new_with_data(index, data))
                    }
                    EntrantSpot::Empty => EntrantSpot::Empty,
                    EntrantSpot::TBD => EntrantSpot::TBD,
                };
            }
        }

        if let Some((entrant, data)) = res.loser {
            if let Some(spot) = next_matches.loser_mut(&mut self.matches) {
                log::debug!("Next loser match is {}", *next_matches.loser_index);

                *spot = match entrant {
                    EntrantSpot::Entrant(index) => {
                        EntrantSpot::Entrant(Node::new_with_data(index, data))
                    }
                    EntrantSpot::Empty => EntrantSpot::Empty,
                    EntrantSpot::TBD => EntrantSpot::TBD,
                };
            }
        }

        let mut next_index = index;
        if res.reset {
            let r#match = self.matches.get_mut(index).unwrap();

            for entrant in r#match.entrants.iter_mut() {
                if let EntrantSpot::Entrant(entrant) = entrant {
                    entrant.data = D::default();
                }
            }

            // Reset all following matches.
            loop {
                let next_matches = self.next_matches(next_index);
                if next_matches.winner_index.is_none() {
                    break;
                }

                // Note: Loser matches don't have any following matches.
                if next_matches.loser_index.is_some() {
                    let r#match = self.matches.get_mut(*next_matches.loser_index).unwrap();
                    r#match[next_matches.loser_position] = EntrantSpot::TBD;
                }

                next_index = *next_matches.winner_index;

                let r#match = self.matches.get_mut(next_index).unwrap();
                r#match[next_matches.winner_position] = EntrantSpot::TBD;
            }
        }
    }

    fn next_matches(&self, index: usize) -> NextMatches {
        let is_final_match = if self.options.third_place_match {
            index >= self.matches().len() - 2
        } else {
            index >= self.matches().len() - 1
        };

        let winner_index = self.entrants.len().next_power_of_two() / 2 + index / 2;
        let loser = if self.options.third_place_match
            && index >= self.matches().len() - 4
            && index != self.matches().len() - 2
        {
            Some((self.matches().len() - 1, index % 2))
        } else {
            None
        };

        if is_final_match {
            NextMatches::default()
        } else {
            NextMatches::new(Some((winner_index, index % 2)), loser)
        }
    }

    #[inline]
    fn next_bracket_round(&self, range: Range<usize>) -> Range<usize> {
        // `range` is `self.matches().len()..self.matches().len()`. No other bracket rounds follow.
        if range.is_empty() {
            range
        } else {
            0..self.matches.len()
        }
    }

    #[inline]
    fn next_bracket(&self, range: Range<usize>) -> Range<usize> {
        // `range` is `self.matches().len()..self.matches().len()`. No other brackets follow.
        if range.is_empty() {
            range
        } else {
            0..self.matches.len()
        }
    }

    #[inline]
    fn next_round(&self, range: Range<usize>) -> Range<usize> {
        // Start from default.
        if range.start == 0 {
            match self.entrants.len() {
                1 => 0..self.entrants().len().next_power_of_two(),
                n => 0..n.next_power_of_two() / 2,
            }
        } else {
            let end = self.entrants().len().next_power_of_two() / 2 + range.start / 2;

            if end == self.matches().len() - 1 && self.options.third_place_match {
                range.start..end + 1
            } else {
                range.start..end
            }
        }
    }

    #[inline]
    fn render_match_position(&self, index: usize) -> Position {
        if self.options.third_place_match
            && self.matches.len() > 2
            && index == self.matches().len() - 1
        {
            Position::bottom(0)
        } else {
            Position::default()
        }
    }
}

impl<T, D> Borrow<Entrants<T>> for SingleElimination<T, D> {
    fn borrow(&self) -> &Entrants<T> {
        &self.entrants
    }
}

impl<T, D> Borrow<Matches<D>> for SingleElimination<T, D> {
    fn borrow(&self) -> &Matches<D> {
        &self.matches
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct SingleEliminationOptions {
    third_place_match: bool,
}

impl SingleEliminationOptions {
    fn new(mut options: TournamentOptionValues) -> Self {
        let mut this = Self::default();

        if let Some(val) = options.take("third_place_match") {
            this.third_place_match = val.unwrap_bool_or(false);
        }

        this
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::TestRenderer;
    use crate::{entrants, option_values};

    use super::*;

    #[test]
    fn test_single_elimination() {
        let entrants = entrants![];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, vec![]);
        assert_eq!(tournament.matches, vec![]);

        // Test with a single entrant.
        let entrants = entrants![0];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, vec![0]);
        assert_eq!(
            tournament.matches,
            vec![Match::new([
                EntrantSpot::Entrant(Node { index: 0, data: 0 }),
                EntrantSpot::Empty
            ])]
        );

        // Test with two entrants.
        let entrants = entrants![0, 1];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, vec![0, 1]);
        assert_eq!(
            tournament.matches,
            vec![Match::new([
                EntrantSpot::Entrant(Node { index: 0, data: 0 }),
                EntrantSpot::Entrant(Node { index: 1, data: 0 })
            ])]
        );

        // Test with three entrants.
        let entrants = entrants![0, 1, 2];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, vec![0, 1, 2]);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(1))]),
            ]
        );

        // Test with pow(2, n) entrants.
        let entrants = entrants![0, 1, 2, 3];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, vec![0, 1, 2, 3]);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
    }

    #[test]
    fn test_single_elimination_third_place_match() {
        let options = option_values!("third_place_match" => true);

        let entrants = entrants![];
        let tournament = SingleElimination::<i32, u32>::new_with_options(entrants, options.clone());

        assert_eq!(tournament.entrants, vec![]);
        assert_eq!(tournament.matches, vec![]);

        let entrants = entrants![1];
        let tournament = SingleElimination::<i32, u32>::new_with_options(entrants, options.clone());

        assert_eq!(tournament.entrants, vec![1]);
        assert_eq!(
            tournament.matches,
            vec![Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Empty,
            ])]
        );

        let entrants = entrants![1, 2];
        let tournament = SingleElimination::<i32, u32>::new_with_options(entrants, options.clone());

        assert_eq!(tournament.entrants, vec![1, 2]);
        assert_eq!(
            tournament.matches,
            vec![Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(1)),
            ])]
        );

        let entrants = entrants![1, 2, 3];
        let tournament = SingleElimination::<i32, u32>::new_with_options(entrants, options.clone());

        assert_eq!(tournament.entrants, vec![1, 2, 3]);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(1))]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        let entrants = entrants![1, 2, 3, 4];
        let tournament = SingleElimination::<i32, u32>::new_with_options(entrants, options);

        assert_eq!(tournament.entrants, vec![1, 2, 3, 4]);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
    }

    #[test]
    fn test_single_elimination_resume() {
        let entrants = Entrants::from(vec![0, 1, 2, 3]);
        let matches = Matches::from(vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(1)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        SingleElimination::<i32, u32>::resume(entrants, matches, TournamentOptions::default())
            .unwrap();

        let entrants = Entrants::from(vec![0, 1, 2, 3, 4]);
        let matches = Matches::from(vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(1)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        assert_eq!(
            SingleElimination::<i32, u32>::resume(entrants, matches, TournamentOptions::default())
                .unwrap_err(),
            Error::InvalidNumberOfMatches {
                expected: 7,
                found: 3
            }
        );

        let entrants = Entrants::from(vec![0, 1, 2, 3]);
        let matches = Matches::from(vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(1)),
                EntrantSpot::Entrant(Node::new(4)),
            ]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        assert_eq!(
            SingleElimination::<i32, u32>::resume(entrants, matches, TournamentOptions::default())
                .unwrap_err(),
            Error::InvalidEntrant {
                index: 4,
                length: 4
            }
        );
    }

    #[test]
    fn test_single_elimination_resume_third_place_match() {
        let options = option_values!("third_place_match" => true);

        let entrants = Entrants::from(vec![]);
        let matches = Matches::from(vec![]);

        SingleElimination::<i32, u32>::resume(entrants, matches, options.clone()).unwrap();

        let entrants = Entrants::from(vec![1]);
        let matches = Matches::from(vec![Match::new([
            EntrantSpot::Entrant(Node::new(0)),
            EntrantSpot::Empty,
        ])]);

        SingleElimination::<i32, u32>::resume(entrants, matches, options.clone()).unwrap();

        let entrants = Entrants::from(vec![1, 2]);
        let matches = Matches::from(vec![Match::new([
            EntrantSpot::Entrant(Node::new(0)),
            EntrantSpot::Entrant(Node::new(1)),
        ])]);

        SingleElimination::<i32, u32>::resume(entrants, matches, options.clone()).unwrap();

        let entrants = Entrants::from(vec![1, 2, 3]);
        let matches = Matches::from(vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([EntrantSpot::Entrant(Node::new(1)), EntrantSpot::Empty]),
            Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(1))]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        SingleElimination::<i32, u32>::resume(entrants, matches, options.clone()).unwrap();

        let entrants = Entrants::from(vec![1, 2, 3, 4]);
        let matches = Matches::from(vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(1)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        SingleElimination::<i32, u32>::resume(entrants, matches, options.clone()).unwrap();

        let entrants = Entrants::from(vec![1, 2, 3]);
        let matches = Matches::from(vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([EntrantSpot::Entrant(Node::new(1)), EntrantSpot::Empty]),
            Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(1))]),
        ]);

        assert_eq!(
            SingleElimination::<i32, u32>::resume(entrants, matches, options).unwrap_err(),
            Error::InvalidNumberOfMatches {
                expected: 4,
                found: 3,
            }
        );
    }

    #[test]
    fn test_single_elimination_update_match() {
        let entrants = entrants![0, 1, 2, 3];
        let mut tournament = SingleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, vec![0, 1, 2, 3]);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        // Update the match at index 0.
        tournament.update_match(0, |r#match, result| {
            result.winner_default(&r#match[0]);
            result.loser_default(&r#match[1]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(0)), EntrantSpot::TBD]),
            ]
        );

        // Update the match at index 1.
        tournament.update_match(1, |r#match, result| {
            result.winner_default(&r#match[1]);
            result.loser_default(&r#match[0]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
            ]
        );

        // Update the match at index 2. The last match won't have any next matches.
        tournament.update_match(2, |r#match, result| {
            result.winner_default(&r#match[0]);
            result.loser_default(&r#match[1]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
            ]
        );
    }

    #[test]
    fn test_single_elimination_reset_match() {
        let entrants = entrants![0, 1, 2, 3].collect();
        let matches = vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(1)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
        ]
        .into();
        let mut tournament = SingleElimination::<i32, u32>::resume(
            entrants,
            matches,
            SingleElimination::<i32, u32>::options(),
        )
        .unwrap();

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
            ]
        );

        tournament.update_match(2, |_, result| {
            result.reset_default();
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
            ]
        );

        tournament.update_match(1, |_, result| {
            result.reset_default();
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(0)), EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(0, |_, result| {
            result.reset_default();
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        let entrants = entrants![0, 1, 2, 3, 4, 5, 6, 7].collect();
        let matches = vec![
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(4)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(1)),
                EntrantSpot::Entrant(Node::new(5)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(2)),
                EntrantSpot::Entrant(Node::new(6)),
            ]),
            Match::new([EntrantSpot::Entrant(Node::new(3)), EntrantSpot::Empty]),
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(5)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(2)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
        ]
        .into();
        let mut tournament =
            SingleElimination::<i32, u32>::resume(entrants, matches, TournamentOptions::default())
                .unwrap();

        assert_eq!(tournament.entrants, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(4)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(3)), EntrantSpot::Empty]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
            ]
        );

        // Reset all matches following index 0.
        tournament.update_match(0, |_, result| {
            result.reset_default();
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(4)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(3)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(5))]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(2))]),
            ]
        );

        // Reset all matches following index 2.
        tournament.update_match(2, |_, result| {
            result.reset_default();
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(4)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(3)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(5))]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(3))]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
    }

    #[test]
    fn test_single_elimination_update_match_third_place_match() {
        let entrants = entrants![0, 1, 2, 3];
        let mut options = SingleElimination::<i32, u32>::options();
        options.set("third_place_match", true);

        let mut tournament = SingleElimination::<i32, u32>::new_with_options(entrants, options);

        assert_eq!(tournament.entrants, vec![0, 1, 2, 3]);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(0, |r#match, result| {
            result.winner_default(&r#match[0]);
            result.loser_default(&r#match[1]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(0)), EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(Node::new(2)), EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(1, |r#match, result| {
            result.winner_default(&r#match[1]);
            result.loser_default(&r#match[0]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
            ]
        );

        // Index 2 is the final match and no changes should happen.
        tournament.update_match(2, |r#match, result| {
            result.winner_default(&r#match[0]);
            result.loser_default(&r#match[1]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
            ]
        );

        // Index 3 is the final match (third place) and no changes should happen.
        tournament.update_match(3, |r#match, result| {
            result.winner_default(&r#match[0]);
            result.loser_default(&r#match[1]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
            ]
        );
    }

    #[test]
    fn test_single_elimination_render() {
        let entrants = entrants![0, 1, 2, 3];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        let mut renderer = TestRenderer::default();
        tournament.render(&mut renderer);

        assert_eq!(
            renderer,
            vec![vec![vec![
                vec![
                    Match::new([
                        EntrantSpot::Entrant(Node::new(0)),
                        EntrantSpot::Entrant(Node::new(2))
                    ]),
                    Match::new([
                        EntrantSpot::Entrant(Node::new(1)),
                        EntrantSpot::Entrant(Node::new(3))
                    ]),
                ],
                vec![Match::new([EntrantSpot::TBD, EntrantSpot::TBD])]
            ]]]
        );
    }
}
