use crate::{Entrant, EntrantRefMut, EntrantSpot, Error, MatchResult, Result};
use crate::{EntrantData, Entrants, Match, Matches, NextMatches};

use std::ptr;

/// A single elimination tournament.
#[derive(Clone, Debug)]
pub struct SingleElimination<T, D> {
    entrants: Entrants<T>,
    matches: Matches<Entrant<D>>,
}

impl<T, D> SingleElimination<T, D>
where
    D: EntrantData + Default,
{
    /// Creates a new `SingleElimination`.
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        let entrants: Entrants<T> = entrants.collect();

        log::debug!(
            "Creating new SingleElimination bracket with {} entrants",
            entrants.len()
        );

        let initial_matches = match entrants.len() {
            1 | 2 => 1,
            n => n.next_power_of_two() / 2,
        };

        let mut matches = Matches::with_capacity(initial_matches * 2 - 1);

        // Push the first half entrants into matches. This already creates the minimum number of
        // matches required.
        let mut ptr = matches.as_mut_ptr();
        for index in 0..initial_matches {
            let first = EntrantSpot::Entrant(Entrant::new(index));
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

            *spot = EntrantSpot::Entrant(Entrant::new(index));
            index += 1;
        }

        // Fill `matches` with `TBD` matches.
        while matches.len() < matches.capacity() {
            matches.push(Match::new([EntrantSpot::TBD, EntrantSpot::TBD]));
        }

        // Forward all placeholder matches.
        while index < entrants.len().next_power_of_two() {
            let new_index = initial_matches + (index - initial_matches) / 2;
            // SAFETY: `new_index` is in bounds of `matches`, `index % 2` never exceeds 1.
            let spot = unsafe {
                matches
                    .get_unchecked_mut(new_index)
                    .get_unchecked_mut(index % 2)
            };

            *spot = EntrantSpot::Entrant(Entrant::new(index - initial_matches));

            index += 1;
        }

        log::debug!(
            "Created new SingleElimination bracket with {} matches",
            matches.len()
        );

        Self { entrants, matches }
    }

    /// Resumes the bracket from existing matches.
    ///
    /// # Errors
    ///
    /// Returns an error if `matches` has an invalid number of matches for `entrants` or an
    /// [`Entrant`] in `matches` pointed to a value that is out-of-bounds.
    pub fn resume(entrants: Entrants<T>, matches: Matches<Entrant<D>>) -> Result<Self> {
        let expected = Self::calculate_matches(entrants.len());
        let found = matches.len();
        if found == expected {
            unsafe { Ok(Self::resume_unchecked(entrants, matches)) }
        } else {
            Err(Error::InvalidNumberOfMatches { expected, found })
        }
    }

    /// Resumes the bracket from existing matches without validating the length of `matches`.
    ///
    /// # Safety
    ///
    /// Calling this function with a number of `matches` that is not valid for the length of
    /// `entrants` will create an [`SingleElimination`] object with false assumptions. Usage
    /// of that invalid object can cause all sorts behavoir including infinite loops, wrong
    /// returned data and potentially undefined behavoir.
    pub unsafe fn resume_unchecked(entrants: Entrants<T>, matches: Matches<Entrant<D>>) -> Self {
        log::debug!(
            "Resuming SingleElimination bracket with {} entrants and {} matches",
            entrants.len(),
            matches.len()
        );

        Self {
            entrants: entrants.into(),
            matches: matches.into(),
        }
    }

    /// Returns a reference to the entrants in the tournament.
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
    pub unsafe fn entrants_mut(&mut self) -> &mut Entrants<T> {
        &mut self.entrants
    }

    /// Returns the entrants from the tournament.
    pub fn into_entrants(self) -> Entrants<T> {
        self.entrants
    }

    /// Returns a reference to the matches in the tournament.
    pub fn matches(&self) -> &Matches<Entrant<D>> {
        &self.matches
    }

    /// Returns a mutable reference to the matches in the tournament.
    ///
    /// # Safety
    ///
    /// [`SingleElimination`] assumes that `matches` has a length of pow(2, n). Violating this
    /// assumption may cause undefined behavoir. Further changing the index field of [`Entrant`]
    /// to a value that is not in bounds of `entrants` causes undefined behavoir.
    ///
    /// Changing the data field of [`Entrant`] without changing the length of [`Matches`] or
    /// changing the index field of [`Entrant`] is always safe, **but may cause the tournament to
    /// be in an incorrect or inconsistent state**.
    pub unsafe fn matches_mut(&mut self) -> &mut Matches<Entrant<D>> {
        &mut self.matches
    }

    /// Returns the matches from the tournament.
    pub fn into_matches(self) -> Matches<Entrant<D>> {
        self.matches
    }

    /// Returns the [`NextMatches`] of the match with the given `index`.
    pub fn next_matches(&self, index: usize) -> NextMatches {
        let winner_index = self.entrants.len().next_power_of_two() / 2 + index / 2;

        if self.matches.len() > winner_index {
            NextMatches::new(Some((winner_index, index % 2)), None)
        } else {
            NextMatches::default()
        }
    }

    /// Updates the match at `index` by applying `f` on it. If the function returns a value other
    /// than [`MatchResult::None`], the next match is updating using the result. If `index` is
    /// out-of-bounds the function is never called.
    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<EntrantRefMut<'_, T, D>>, &mut MatchResult<D>),
    {
        // Get the match at `index` or abort.
        // Note: This will borrow `self.matches` mutably until the end of the scope. All
        // operations that access `self.matches` at an index that is **not `index`** are still
        // safe.

        let mut r#match = match self.matches.get_mut(index) {
            Some(r#match) => r#match.to_ref_mut(&self.entrants),
            None => return,
        };

        let mut res = MatchResult::default();

        f(&mut r#match, &mut res);

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
                        EntrantSpot::Entrant(Entrant::new_with_data(index, data))
                    }
                    EntrantSpot::Empty => EntrantSpot::Empty,
                    EntrantSpot::TBD => EntrantSpot::TBD,
                };
            }
        }

        if res.reset {
            let r#match = self.matches.get_mut(index).unwrap();

            for entrant in r#match.entrants.iter_mut() {
                if let EntrantSpot::Entrant(entrant) = entrant {
                    entrant.data = D::default();
                }
            }
        }
    }

    pub fn rounds_iter(&self) -> RoundsIter<'_, Entrant<D>> {
        RoundsIter::new(
            self.matches.as_ref(),
            match self.entrants.len() {
                1 | 2 => 1,
                n => n.next_power_of_two() / 2,
            },
        )
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

impl<T, D> AsRef<Entrants<T>> for SingleElimination<T, D> {
    fn as_ref(&self) -> &Entrants<T> {
        &self.entrants
    }
}

impl<T, D> AsRef<Matches<Entrant<D>>> for SingleElimination<T, D> {
    fn as_ref(&self) -> &Matches<Entrant<D>> {
        &self.matches
    }
}

#[derive(Debug)]
pub struct RoundsIter<'a, T> {
    slice: &'a [Match<T>],
    num_matches: usize,
}

impl<'a, T> RoundsIter<'a, T> {
    fn new(slice: &'a [Match<T>], num_matches: usize) -> Self {
        Self { slice, num_matches }
    }

    pub fn with_index(self) -> RoundsIterIndex<'a, T> {
        RoundsIterIndex {
            slice: self.slice,
            num_matches: self.num_matches,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for RoundsIter<'a, T> {
    type Item = &'a [Match<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            // TODO: This can be `split_at_unchecked` when it is stable.
            let (slice, rem) = self.slice.split_at(self.num_matches);

            self.slice = rem;
            self.num_matches /= 2;

            Some(slice)
        }
    }
}

pub struct RoundsIterIndex<'a, T> {
    slice: &'a [Match<T>],
    num_matches: usize,
    index: usize,
}

impl<'a, T> Iterator for RoundsIterIndex<'a, T> {
    type Item = (usize, &'a [Match<T>]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (slice, rem) = self.slice.split_at(self.num_matches);
            let index = self.index;

            self.slice = rem;
            self.index += slice.len();
            self.num_matches /= 2;

            Some((index, slice))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entrants;

    use super::*;

    #[test]
    fn test_single_elimination() {
        // Test with a single entrant.
        let entrants = entrants![0];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, vec![0]);
        assert_eq!(
            tournament.matches,
            vec![Match::new([
                EntrantSpot::Entrant(Entrant { index: 0, data: 0 }),
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
                EntrantSpot::Entrant(Entrant { index: 0, data: 0 }),
                EntrantSpot::Entrant(Entrant { index: 1, data: 0 })
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([EntrantSpot::Entrant(Entrant::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Entrant::new(1))]),
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(1)),
                    EntrantSpot::Entrant(Entrant::new(3))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(1)),
                    EntrantSpot::Entrant(Entrant::new(3))
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(1)),
                    EntrantSpot::Entrant(Entrant::new(3))
                ]),
                Match::new([EntrantSpot::Entrant(Entrant::new(0)), EntrantSpot::TBD]),
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(1)),
                    EntrantSpot::Entrant(Entrant::new(3))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(3))
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(1)),
                    EntrantSpot::Entrant(Entrant::new(3))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(3))
                ]),
            ]
        );
    }

    #[test]
    fn test_single_elimination_rounds_iter() {
        let entrants = entrants![0, 1, 2, 3];
        let tournament = SingleElimination::<i32, u32>::new(entrants);

        let mut iter = tournament.rounds_iter();
        assert_eq!(
            iter.next().unwrap(),
            [
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(1)),
                    EntrantSpot::Entrant(Entrant::new(3))
                ]),
            ]
        );

        assert_eq!(
            iter.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),]
        );

        assert_eq!(iter.next(), None);
    }
}
