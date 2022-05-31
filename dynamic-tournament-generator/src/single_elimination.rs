use crate::options::TournamentOptions;
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
    options: TournamentOptions,
}

impl<T, D> SingleElimination<T, D>
where
    D: EntrantData + Default,
{
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        Self::new_with_options(entrants, Self::options())
    }

    /// Creates a new `SingleElimination`.
    pub fn new_with_options<I>(entrants: I, options: TournamentOptions) -> Self
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

        let mut num_matches = (initial_matches * 2).saturating_sub(1);
        if let Some(opt) = options.get("third_place_match") {
            if opt.value.unwrap_bool() {
                num_matches += 1;
            }
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
    /// [`Entrant`] in `matches` pointed to a value that is out-of-bounds.
    pub fn resume(
        entrants: Entrants<T>,
        matches: Matches<D>,
        options: TournamentOptions,
    ) -> Result<Self> {
        let expected = Self::calculate_matches(entrants.len());
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
    pub unsafe fn resume_unchecked(
        entrants: Entrants<T>,
        matches: Matches<D>,
        options: TournamentOptions,
    ) -> Self {
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
    /// assumption may cause undefined behavoir. Further changing the index field of [`Entrant`]
    /// to a value that is not in bounds of `entrants` causes undefined behavoir.
    ///
    /// Changing the data field of [`Entrant`] without changing the length of [`Matches`] or
    /// changing the index field of [`Entrant`] is always safe, **but may cause the tournament to
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

    /// Returns the [`NextMatches`] of the match with the given `index`.
    pub fn next_matches(&self, index: usize) -> NextMatches {
        let winner_index = self.entrants.len().next_power_of_two() / 2 + index / 2;

        if self.matches.len() > winner_index {
            NextMatches::new(Some((winner_index, index % 2)), None)
        } else {
            NextMatches::default()
        }
    }

    /// Updates the match at `index` by applying `f` on it. The next match is updating using the
    /// result. If `index` is out-of-bounds the function is never called.
    pub fn update_match<F>(&mut self, index: usize, f: F)
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

                next_index = *next_matches.winner_index;

                let r#match = self.matches.get_mut(next_index).unwrap();

                r#match[next_matches.winner_position] = EntrantSpot::TBD;
            }
        }
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

        let mut r#match = match self.matches.get_mut(index) {
            Some(r#match) => r#match,
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
                        EntrantSpot::Entrant(Node::new_with_data(index, data))
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

    fn next_matches(&self, index: usize) -> NextMatches {
        let winner_index = self.entrants.len().next_power_of_two() / 2 + index / 2;

        if self.matches.len() > winner_index {
            NextMatches::new(Some((winner_index, index % 2)), None)
        } else {
            NextMatches::default()
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

    fn next_round(&self, range: Range<usize>) -> Range<usize> {
        // Start from default.
        if range.start == 0 {
            match self.entrants.len() {
                1 => 0..self.entrants().len().next_power_of_two(),
                n => 0..n.next_power_of_two() / 2,
            }
        } else {
            range.start..self.entrants().len().next_power_of_two() / 2 + range.start / 2
        }
    }

    fn render_match_position(&self, index: usize) -> Position {
        if let Some(opt) = self.options.get("third_place_match") {
            if opt.value.unwrap_bool() && index == self.matches().len() - 1 {
                return Position::bottom(0);
            }
        }

        Position::default()
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

#[cfg(test)]
mod tests {
    use crate::entrants;
    use crate::tests::TestRenderer;

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
