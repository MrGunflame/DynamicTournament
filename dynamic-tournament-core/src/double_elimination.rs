use crate::{
    render::{Column, Element, ElementInner, Position, RenderState, Row},
    EntrantData, EntrantSpot, Entrants, Error, Match, MatchResult, Matches, NextMatches, Node,
    Result, System,
};

use std::{borrow::Borrow, marker::PhantomData};

/// A double elimination tournament.
#[derive(Clone, Debug)]
pub struct DoubleElimination<T, D>
where
    D: EntrantData,
{
    entrants: Entrants<T>,
    matches: Matches<D>,
    lower_bracket_index: usize,
}

impl<T, D> DoubleElimination<T, D>
where
    D: EntrantData,
{
    /// Creates a new `DoubleElimination` tournament with the given `entrants`.
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        let entrants: Entrants<T> = entrants.collect();

        log::debug!(
            "Creating a new DoubleElimination bracket with {} entrants",
            entrants.len()
        );

        if entrants.len() == 0 {
            return Self {
                entrants: Entrants::new(),
                matches: Matches::new(),
                lower_bracket_index: 0,
            };
        }

        let initial_matches = match entrants.len() {
            1 | 2 => 1,
            n => n.next_power_of_two() / 2,
        };

        // The upper bracket has exactly initial_matches * 2 - 1 matches, the lower bracket has
        // exactly the matches of the upper bracket - 1 matches (or initial_matches * 2 - 2).
        // Plus one additional match for the final bracket: `(initial_matches * 2 - 1) +
        // (initial_matches * 2 - 2) + 1 = initial_matches * 4 - 2`.
        let mut matches = Matches::with_capacity(match entrants.len() {
            1 | 2 => 1,
            _ => initial_matches * 4 - 2,
        });

        // This is out-of-bounds for brackets with one match, but it doesn't matter as it's never
        // used in that case.
        let lower_bracket_index = initial_matches * 2 - 1;

        for index in 0..initial_matches {
            let first = EntrantSpot::Entrant(Node::new(index));
            let second = EntrantSpot::Empty;

            matches.push(Match::new([first, second]));
        }

        let mut index = initial_matches;
        while index < entrants.len() {
            let spot = matches
                .get_mut(index - initial_matches)
                .unwrap()
                .get_mut(1)
                .unwrap();

            *spot = EntrantSpot::Entrant(Node::new(index));
            index += 1;
        }

        while matches.len() < matches.capacity() {
            matches.push(Match::new([EntrantSpot::TBD, EntrantSpot::TBD]));
        }

        // Forward all placeholder matches.
        while index < entrants.len().next_power_of_two() {
            // Upper bracket:
            let new_index = initial_matches + (index - initial_matches) / 2;

            let spot = unsafe {
                matches
                    .get_unchecked_mut(new_index)
                    .get_unchecked_mut(index % 2)
            };

            *spot = EntrantSpot::Entrant(Node::new(index - initial_matches));

            // Lower bracket
            let new_index = (index - initial_matches) / 2 + lower_bracket_index;

            let spot = unsafe {
                matches
                    .get_unchecked_mut(new_index)
                    .get_unchecked_mut(index % 2)
            };

            *spot = EntrantSpot::Empty;

            index += 1;
        }

        log::debug!(
            "Created a new DoubleElimination bracket with {} matches",
            matches.len()
        );

        Self {
            entrants,
            matches,
            lower_bracket_index,
        }
    }

    /// Resumes the bracket from existing matches.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if `matches` has an invalid number of matches for `entrants` or an
    /// [`Node`] in `matches` pointed to a value that is out-of-bounds.
    pub fn resume(entrants: Entrants<T>, matches: Matches<D>) -> Result<Self> {
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
        unsafe { Ok(Self::resume_unchecked(entrants, matches)) }
    }

    /// Resumes the bracket from existing matches without validating the length of `matches`.
    ///
    /// # Safety
    ///
    /// Calling this function with a number of `matches` that is not valid for the length of
    /// `entrants` will create an [`DoubleElimination`] object with false assumptions. Usage
    /// of that invalid object can cause all sorts behavoir including infinite loops, wrong
    /// returned data and potentially undefined behavoir.
    pub unsafe fn resume_unchecked(entrants: Entrants<T>, matches: Matches<D>) -> Self {
        log::debug!(
            "Resuming DoubleElimination bracket with {} entrants and {} matches",
            entrants.len(),
            matches.len()
        );

        let lower_bracket_index = matches.len() / 2;

        Self {
            entrants,
            matches,
            lower_bracket_index,
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
    /// [`DoubleElimination`] generally assumes that `entrants` has a correct length and capacity
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
    pub fn matches(&self) -> &Matches<D> {
        &self.matches
    }

    /// Returns a mutable reference to matches in the tournament.
    ///
    /// # Safety
    ///
    /// [`DoubleElimination`] assumes that `matches` has a length of
    /// `self.entrants.len().next_power_of_two() * 2 - 1`. Violating this assumption may cause
    /// undefined behavoir. Further changing the `index` field of [`Node`] to a value that is
    /// not in bounds of `entrants` causes undefined behavoir.
    ///
    /// Changing the data field of [`Node`] without changing the length of [`Matches`] or
    /// changing the index field of [`Node`] is always safe, **but may cause the tournament to
    /// be in an incorrect or inconsistent state**.
    pub unsafe fn matches_mut(&mut self) -> &mut Matches<D> {
        &mut self.matches
    }

    /// Returns the matches from the tournament.
    pub fn into_matches(self) -> Matches<D> {
        self.matches
    }

    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<D>>, &mut MatchResult<D>),
    {
        log::debug!("Updating match {}", index);

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
            if let Some(spot) = next_matches.winner_mut(&mut self.matches) {
                log::debug!(
                    "Next winner match is {}",
                    next_matches.winner_index().unwrap()
                );

                *spot = entrant.map(|index| Node::new_with_data(index, data));
            }
        }

        if let Some((entrant, data)) = res.loser {
            if let Some(m) = next_matches.loser_match_mut(&mut self.matches) {
                log::debug!(
                    "Next loser match is {}",
                    next_matches.loser_index().unwrap()
                );

                let mut index = 0;
                let entrant = entrant.map(|i| {
                    index = i;
                    Node::new_with_data(index, data)
                });

                unsafe {
                    *m.get_unchecked_mut(next_matches.loser_position().unwrap()) = entrant;
                }

                if m.is_placeholder() {
                    unsafe {
                        if let EntrantSpot::Entrant(entrant) =
                            m.get_unchecked_mut(next_matches.loser_position().unwrap())
                        {
                            entrant.data.set_winner(true);
                        }
                    }

                    let next_matches = self.next_matches(next_matches.loser_index().unwrap());

                    if let Some(spot) = next_matches.winner_mut(&mut self.matches) {
                        *spot = EntrantSpot::Entrant(Node::new(index));
                    }
                }
            }
        }
    }

    pub fn next_matches(&self, index: usize) -> NextMatches {
        // The number of matches in the first round of the upper bracket.
        let initial_matches = self.entrants.len().next_power_of_two() / 2;

        match index {
            // Final match or out-of-bounds: no next matches.
            i if i >= self.final_bracket_index() => NextMatches::default(),
            // Lower bracket match
            i if i >= self.lower_bracket_index => {
                let mut round_index = 0;
                let mut buffer = 0;
                let mut num_matches = initial_matches / 2;
                while index - self.lower_bracket_index >= buffer + num_matches {
                    round_index += 1;
                    buffer += num_matches;

                    if round_index % 2 == 0 {
                        num_matches /= 2;
                    }
                }

                let winner = index - buffer - self.lower_bracket_index;

                let (winner, position) = match round_index {
                    i if i == self.final_bracket_index() - 1 => (self.final_bracket_index(), 1),
                    i if i % 2 == 0 => (index + num_matches, 0),
                    _ => (index + (num_matches - winner + winner / 2), (index - 1) % 2),
                };

                NextMatches::new(Some((winner, position)), None)
            }
            // Upper bracket match
            i => match i {
                // Final match in the upper bracket: Move the winner to the final bracket (spot 1)
                // and the loser to the last match in the lower bracket (spot 2).
                i if i == self.lower_bracket_index - 1 => {
                    let winner_index = self.final_bracket_index();
                    let loser_index = self.final_bracket_index() - 1;

                    NextMatches::new(Some((winner_index, 0)), Some((loser_index, 1)))
                }
                // The first round of matches. All matches in the lower bracket need to be filled.
                i if i < initial_matches => {
                    let winner_index = initial_matches + i / 2;
                    let loser_index = self.lower_bracket_index + (i / 2);

                    NextMatches::new(
                        Some((winner_index, index % 2)),
                        Some((loser_index, index % 2)),
                    )
                }
                index => {
                    let winner_index = initial_matches + index / 2;

                    // Find the index of the match in second round of the lower bracket with the
                    // same number of matches as in the current round.
                    let mut buffer = initial_matches;
                    let mut num_matches = initial_matches / 2;
                    let mut lower_buffer = 0;
                    while index - self.upper_match_index(index) >= buffer {
                        buffer += num_matches;
                        lower_buffer += num_matches * 2;
                        num_matches /= 2;
                    }

                    let loser_index =
                        self.lower_bracket_index + lower_buffer + self.upper_match_index(index)
                            - num_matches * 2;

                    NextMatches::new(Some((winner_index, index % 2)), Some((loser_index, 1)))
                }
            },
        }
    }

    /// Returns the index of the starting match of the final bracket.
    fn final_bracket_index(&self) -> usize {
        self.matches.len().saturating_sub(1)
    }

    fn upper_match_index(&self, index: usize) -> usize {
        let mut buffer = 0;
        let mut start = self.entrants.len().next_power_of_two();
        while index >= buffer + start {
            buffer += start;
            start /= 2;
        }

        index - buffer
    }

    /// Calculates the number of matches required to build a [`DoubleElimination`] tournament
    /// using `entrants`-number of entrants.
    fn calculate_matches(entrants: usize) -> usize {
        match entrants {
            1 | 2 => 1,
            n => n.next_power_of_two() * 2 - 2,
        }
    }
}

impl<T, D> System for DoubleElimination<T, D>
where
    D: EntrantData + Default,
{
    type Entrant = T;
    type NodeData = D;

    fn entrants(&self) -> &Entrants<T> {
        &self.entrants
    }

    unsafe fn entrants_mut(&mut self) -> &mut Entrants<T> {
        &mut self.entrants
    }

    fn into_entrants(self) -> Entrants<T> {
        self.entrants
    }

    fn matches(&self) -> &Matches<D> {
        &self.matches
    }

    unsafe fn matches_mut(&mut self) -> &mut Matches<D> {
        &mut self.matches
    }

    fn into_matches(self) -> Matches<D> {
        self.matches
    }

    fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<D>>, &mut MatchResult<D>),
    {
        log::debug!("Updating match {}", index);

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
            if let Some(spot) = next_matches.winner_mut(&mut self.matches) {
                log::debug!(
                    "Next winner match is {}",
                    next_matches.winner_index().unwrap()
                );

                *spot = entrant.map(|index| Node::new_with_data(index, data));
            }
        }

        if let Some((entrant, data)) = res.loser {
            if let Some(m) = next_matches.loser_match_mut(&mut self.matches) {
                log::debug!(
                    "Next loser match is {}",
                    next_matches.loser_index().unwrap()
                );

                let mut index = 0;
                let entrant = entrant.map(|i| {
                    index = i;
                    Node::new_with_data(index, data)
                });

                unsafe {
                    *m.get_unchecked_mut(next_matches.loser_position().unwrap()) = entrant;
                }

                if m.is_placeholder() {
                    unsafe {
                        if let EntrantSpot::Entrant(entrant) =
                            m.get_unchecked_mut(next_matches.loser_position().unwrap())
                        {
                            entrant.data.set_winner(true);
                        }
                    }

                    let next_matches = self.next_matches(next_matches.loser_index().unwrap());

                    if let Some(spot) = next_matches.winner_mut(&mut self.matches) {
                        *spot = EntrantSpot::Entrant(Node::new(index));
                    }
                }
            }
        }
    }

    fn next_matches(&self, index: usize) -> NextMatches {
        // The number of matches in the first round of the upper bracket.
        let initial_matches = self.entrants.len().next_power_of_two() / 2;

        match index {
            // Final match or out-of-bounds: no next matches.
            i if i >= self.final_bracket_index() => NextMatches::default(),
            // Lower bracket match
            i if i >= self.lower_bracket_index => {
                let mut round_index = 0;
                let mut buffer = 0;
                let mut num_matches = initial_matches / 2;
                while index - self.lower_bracket_index >= buffer + num_matches {
                    round_index += 1;
                    buffer += num_matches;

                    if round_index % 2 == 0 {
                        num_matches /= 2;
                    }
                }

                let winner = index - buffer - self.lower_bracket_index;

                let (winner, position) = match round_index {
                    i if i == self.final_bracket_index() - 1 => (self.final_bracket_index(), 1),
                    i if i % 2 == 0 => (index + num_matches, 0),
                    _ => (index + (num_matches - winner + winner / 2), (index - 1) % 2),
                };

                NextMatches::new(Some((winner, position)), None)
            }
            // Upper bracket match
            i => match i {
                // Final match in the upper bracket: Move the winner to the final bracket (spot 1)
                // and the loser to the last match in the lower bracket (spot 2).
                i if i == self.lower_bracket_index - 1 => {
                    let winner_index = self.final_bracket_index();
                    let loser_index = self.final_bracket_index() - 1;

                    NextMatches::new(Some((winner_index, 0)), Some((loser_index, 1)))
                }
                // The first round of matches. All matches in the lower bracket need to be filled.
                i if i < initial_matches => {
                    let winner_index = initial_matches + i / 2;
                    let loser_index = self.lower_bracket_index + (i / 2);

                    NextMatches::new(
                        Some((winner_index, index % 2)),
                        Some((loser_index, index % 2)),
                    )
                }
                index => {
                    let winner_index = initial_matches + index / 2;

                    // Find the index of the match in second round of the lower bracket with the
                    // same number of matches as in the current round.
                    let mut buffer = initial_matches;
                    let mut num_matches = initial_matches / 2;
                    let mut lower_buffer = 0;
                    while index - self.upper_match_index(index) >= buffer {
                        buffer += num_matches;
                        lower_buffer += num_matches * 2;
                        num_matches /= 2;
                    }

                    let loser_index =
                        self.lower_bracket_index + lower_buffer + self.upper_match_index(index)
                            - num_matches * 2;

                    NextMatches::new(Some((winner_index, index % 2)), Some((loser_index, 1)))
                }
            },
        }
    }

    fn start_render(&self) -> RenderState<'_, Self> {
        let initial_matches = self.entrants.len().next_power_of_two() / 2;

        let mut columns = Vec::new();
        // Upper and lower brackets
        // Upper bracket (same impl as single elims)
        let upper = {
            let mut cols = Vec::new();
            let mut num_matches = initial_matches;
            let mut index = 0;

            while num_matches > 0 {
                let mut matches = Vec::new();
                for i in index..index + num_matches {
                    matches.push(Element::new(crate::render::Match {
                        index: i,
                        predecessors: vec![],
                        _marker: PhantomData,
                    }));
                }

                cols.push(Element {
                    label: None,
                    position: Some(Position::SpaceAround),
                    inner: ElementInner::Column(Column::new(matches)),
                });

                index += num_matches;
                num_matches /= 2;
            }

            cols
        };

        let lower = {
            let mut cols = Vec::new();
            let mut num_matches = initial_matches / 2;
            let mut index = self.lower_bracket_index;
            let mut round_index = 0;

            while num_matches > 0 {
                round_index += 1;

                let mut matches = Vec::new();
                for i in index..index + num_matches {
                    matches.push(Element::new(crate::render::Match {
                        index: i,
                        predecessors: vec![],
                        _marker: PhantomData,
                    }));
                }

                cols.push(Element {
                    label: None,
                    position: Some(Position::SpaceAround),
                    inner: ElementInner::Column(Column::new(matches)),
                });

                index += num_matches;
                if round_index % 2 == 0 {
                    num_matches /= 2;
                }
            }

            cols
        };

        columns.push(Element {
            position: Some(Position::SpaceAround),
            inner: ElementInner::Column(Column::new(vec![
                Element {
                    position: Some(Position::SpaceAround),
                    inner: ElementInner::Row(Row::new(upper)),
                    label: None,
                },
                Element {
                    position: Some(Position::SpaceAround),
                    inner: ElementInner::Row(Row::new(lower)),
                    label: None,
                },
            ])),
            label: None,
        });

        // Final match
        columns.push(Element {
            label: None,
            position: Some(Position::SpaceAround),
            inner: ElementInner::Column(Column::new(vec![Element {
                label: None,
                position: None,
                inner: ElementInner::Match(crate::render::Match {
                    index: self.matches.len() - 1,
                    predecessors: vec![],
                    _marker: PhantomData,
                }),
            }])),
        });

        RenderState {
            root: Element::new(Row::new(columns)),
        }
    }
}

impl<T, D> Borrow<Entrants<T>> for DoubleElimination<T, D>
where
    D: EntrantData,
{
    fn borrow(&self) -> &Entrants<T> {
        self.entrants()
    }
}

impl<T, D> Borrow<Matches<D>> for DoubleElimination<T, D>
where
    D: EntrantData,
{
    fn borrow(&self) -> &Matches<D> {
        self.matches()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entrants,
        tests::{TColumn, TElement, TMatch, TRow, TestRenderer},
    };

    use super::*;

    #[test]
    fn test_double_elimination() {
        let entrants = entrants![0];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0]);
        assert_eq!(tournament.lower_bracket_index, 1);
        assert_eq!(
            tournament.matches,
            [Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Empty,
            ])]
        );

        let entrants = entrants![0, 1];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1]);
        assert_eq!(tournament.lower_bracket_index, 1);
        assert_eq!(
            tournament.matches,
            [Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(1))
            ])]
        );

        let entrants = entrants![0, 1, 2];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2]);
        assert_eq!(tournament.lower_bracket_index, 3);
        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(1))]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        let entrants = entrants![0, 1, 2, 3];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2, 3]);
        assert_eq!(tournament.lower_bracket_index, 3);
        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        let entrants = entrants![0, 1, 2, 3, 4];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2, 3, 4]);
        assert_eq!(tournament.lower_bracket_index, 7);
        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(4))
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(Node::new(2)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(Node::new(3)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(1))]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Empty]),
                Match::new([EntrantSpot::Empty, EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
    }

    #[test]
    fn test_double_elimination_resume() {
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
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        DoubleElimination::<i32, u32>::resume(entrants, matches).unwrap();

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
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        assert_eq!(
            DoubleElimination::<i32, u32>::resume(entrants, matches).unwrap_err(),
            Error::InvalidNumberOfMatches {
                expected: 14,
                found: 6
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
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
        ]);

        assert_eq!(
            DoubleElimination::<i32, u32>::resume(entrants, matches).unwrap_err(),
            Error::InvalidEntrant {
                index: 4,
                length: 4
            }
        );
    }

    #[test]
    fn test_double_elimination_update_match() {
        let entrants = entrants![0, 1, 2, 3];
        let mut tournament = DoubleElimination::<i32, u32>::new(entrants);

        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(0, |m, res| {
            res.winner_default(&m[0]);
            res.loser_default(&m[1]);
        });
        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(0)), EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(Node::new(2)), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(1, |m, res| {
            res.winner_default(&m[1]);
            res.loser_default(&m[0]);
        });
        assert_eq!(
            tournament.matches,
            [
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
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(2, |m, res| {
            res.winner_default(&m[0]);
            res.loser_default(&m[1]);
        });
        assert_eq!(
            tournament.matches,
            [
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
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1))
                ]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Node::new(3))]),
                Match::new([EntrantSpot::Entrant(Node::new(0)), EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(3, |m, res| {
            res.winner_default(&m[0]);
            res.loser_default(&m[1]);
        });
        assert_eq!(
            tournament.matches,
            [
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
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([EntrantSpot::Entrant(Node::new(0)), EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(4, |m, res| {
            res.winner_default(&m[1]);
            res.loser_default(&m[0]);
        });
        assert_eq!(
            tournament.matches,
            [
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
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3))
                ]),
            ]
        );

        tournament.update_match(5, |m, res| {
            res.winner_default(&m[1]);
            res.loser_default(&m[0]);
        });
        assert_eq!(
            tournament.matches,
            [
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
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(1))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
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
    fn test_double_elimination_render() {
        let entrants = entrants![0, 1, 2, 3];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        let mut renderer = TestRenderer::new();
        tournament.render(&mut renderer);

        assert_eq!(
            renderer,
            TElement::Row(TRow(vec![
                TElement::Column(TColumn(vec![
                    TElement::Row(TRow(vec![
                        TElement::Column(TColumn(vec![
                            TElement::Match(TMatch { index: 0 }),
                            TElement::Match(TMatch { index: 1 }),
                        ])),
                        TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 2 })])),
                    ]),),
                    TElement::Row(TRow(vec![
                        TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 3 })])),
                        TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 4 })])),
                    ])),
                ])),
                TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 5 }),]))
            ]))
        );

        let entrants = entrants![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        let mut renderer = TestRenderer::new();
        tournament.render(&mut renderer);

        assert_eq!(
            renderer,
            TElement::Row(TRow(vec![
                TElement::Column(TColumn(vec![
                    TElement::Row(TRow(vec![
                        TElement::Column(TColumn(vec![
                            TElement::Match(TMatch { index: 0 }),
                            TElement::Match(TMatch { index: 1 }),
                            TElement::Match(TMatch { index: 2 }),
                            TElement::Match(TMatch { index: 3 }),
                        ])),
                        TElement::Column(TColumn(vec![
                            TElement::Match(TMatch { index: 4 }),
                            TElement::Match(TMatch { index: 5 }),
                        ])),
                        TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 6 }),])),
                    ])),
                    TElement::Row(TRow(vec![
                        TElement::Column(TColumn(vec![
                            TElement::Match(TMatch { index: 7 }),
                            TElement::Match(TMatch { index: 8 }),
                        ])),
                        TElement::Column(TColumn(vec![
                            TElement::Match(TMatch { index: 9 }),
                            TElement::Match(TMatch { index: 10 }),
                        ])),
                        TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 11 })])),
                        TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 12 })])),
                    ]))
                ])),
                TElement::Column(TColumn(vec![TElement::Match(TMatch { index: 13 }),])),
            ]))
        );
    }
}
