use crate::{
    Entrant, EntrantData, EntrantRefMut, EntrantSpot, Entrants, Match, MatchResult, Matches,
    NextMatches,
};

/// A double elimination tournament.
pub struct DoubleElimination<T, D>
where
    D: EntrantData,
{
    entrants: Entrants<T>,
    matches: Matches<Entrant<D>>,
    lower_bracket_index: usize,
}

impl<T, D> DoubleElimination<T, D>
where
    D: EntrantData,
{
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        let entrants: Entrants<T> = entrants.collect();

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
            let first = EntrantSpot::Entrant(Entrant::new(index));
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

            *spot = EntrantSpot::Entrant(Entrant::new(index));
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

            *spot = EntrantSpot::Entrant(Entrant::new(index - initial_matches));

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

        Self {
            entrants,
            matches,
            lower_bracket_index,
        }
    }

    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<EntrantRefMut<'_, T, D>>, &mut MatchResult<D>),
    {
        let mut match_ = match self.matches.get_mut(index) {
            Some(match_) => match_.to_ref_mut(&self.entrants),
            None => return,
        };

        let mut res = MatchResult::default();

        f(&mut match_, &mut res);

        let next_matches = self.next_matches(index);

        if let Some((entrant, data)) = res.winner {
            if let Some(spot) = next_matches.winner_mut(&mut self.matches) {
                *spot = entrant.map(|index| Entrant::new_with_data(index, data));
            }
        }

        if let Some((entrant, data)) = res.loser {
            if let Some(m) = next_matches.loser_match_mut(&mut self.matches) {
                let entrant = entrant.map(|index| Entrant::new_with_data(index, data));

                unsafe {
                    *m.get_unchecked_mut(next_matches.loser_position % 2) = entrant;
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
                    let winner_index = self.entrants.len().next_power_of_two() + i / 2;
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

    pub fn upper_bracket_iter(&self) -> UpperBracketIter<'_, Entrant<D>> {
        UpperBracketIter {
            slice: &self.matches[0..self.lower_bracket_index],
            num_matches: self.entrants.len().next_power_of_two() / 2,
        }
    }

    pub fn lower_bracket_iter(&self) -> LowerBracketIter<'_, Entrant<D>> {
        LowerBracketIter {
            slice: &self.matches[self.lower_bracket_index..self.final_bracket_index()],
            num_matches: self.entrants.len().next_power_of_two() / 4,
            index: 0,
            starting_index: self.lower_bracket_index,
        }
    }

    pub fn final_bracket_iter(&self) -> FinalBracketIter<'_, Entrant<D>> {
        FinalBracketIter {
            slice: &self.matches[self.final_bracket_index()..],
            starting_index: self.final_bracket_index(),
        }
    }
}

#[derive(Debug)]
pub struct UpperBracketIter<'a, T> {
    slice: &'a [Match<T>],
    num_matches: usize,
}

impl<'a, T> UpperBracketIter<'a, T> {
    pub fn with_index(self) -> UpperBracketIndexIter<'a, T> {
        UpperBracketIndexIter {
            slice: self.slice,
            num_matches: self.num_matches,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for UpperBracketIter<'a, T> {
    type Item = &'a [Match<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (slice, rem) = self.slice.split_at(self.num_matches);

            self.slice = rem;
            self.num_matches /= 2;

            Some(slice)
        }
    }
}

#[derive(Debug)]
pub struct UpperBracketIndexIter<'a, T> {
    slice: &'a [Match<T>],
    num_matches: usize,
    index: usize,
}

impl<'a, T> Iterator for UpperBracketIndexIter<'a, T> {
    type Item = (usize, &'a [Match<T>]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (slice, rem) = self.slice.split_at(self.num_matches);
            let index = self.index;

            self.slice = rem;
            self.num_matches /= 2;
            self.index += slice.len();

            Some((index, slice))
        }
    }
}

#[derive(Debug)]
pub struct LowerBracketIter<'a, T> {
    slice: &'a [Match<T>],
    num_matches: usize,
    index: usize,
    starting_index: usize,
}

impl<'a, T> LowerBracketIter<'a, T> {
    pub fn with_index(self) -> LowerBracketIndexIter<'a, T> {
        LowerBracketIndexIter {
            slice: self.slice,
            num_matches: self.num_matches,
            index: self.index,
            starting_index: self.starting_index,
        }
    }
}

impl<'a, T> Iterator for LowerBracketIter<'a, T> {
    type Item = &'a [Match<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (slice, rem) = self.slice.split_at(self.num_matches);

            self.slice = rem;
            self.index += 1;

            if self.index % 2 == 0 {
                self.num_matches /= 2;
            }

            Some(slice)
        }
    }
}

#[derive(Debug)]
pub struct LowerBracketIndexIter<'a, T> {
    slice: &'a [Match<T>],
    num_matches: usize,
    index: usize,
    starting_index: usize,
}

impl<'a, T> Iterator for LowerBracketIndexIter<'a, T> {
    type Item = (usize, &'a [Match<T>]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (slice, rem) = self.slice.split_at(self.num_matches);
            let index = self.starting_index;

            self.slice = rem;
            self.index += 1;
            self.starting_index += slice.len();

            if self.index % 2 == 0 {
                self.num_matches /= 2;
            }

            Some((index, slice))
        }
    }
}

#[derive(Debug)]
pub struct FinalBracketIter<'a, T> {
    slice: &'a [Match<T>],
    starting_index: usize,
}

impl<'a, T> FinalBracketIter<'a, T> {
    pub fn with_index(self) -> FinalBracketIndexIter<'a, T> {
        FinalBracketIndexIter {
            slice: self.slice,
            starting_index: self.starting_index,
        }
    }
}

impl<'a, T> Iterator for FinalBracketIter<'a, T> {
    type Item = &'a [Match<T>];

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (slice, rem) = self.slice.split_at(1);

            self.slice = rem;

            Some(slice)
        }
    }
}

#[derive(Debug)]
pub struct FinalBracketIndexIter<'a, T> {
    slice: &'a [Match<T>],
    starting_index: usize,
}

impl<'a, T> Iterator for FinalBracketIndexIter<'a, T> {
    type Item = (usize, &'a [Match<T>]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.slice.is_empty() {
            None
        } else {
            let (slice, rem) = self.slice.split_at(1);
            let index = self.starting_index;

            self.slice = rem;
            self.starting_index += 1;

            Some((index, slice))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::entrants;

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
                EntrantSpot::Entrant(Entrant::new(0)),
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
                EntrantSpot::Entrant(Entrant::new(0)),
                EntrantSpot::Entrant(Entrant::new(1))
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([EntrantSpot::Entrant(Entrant::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Entrant::new(1))]),
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(2))
                ]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(1)),
                    EntrantSpot::Entrant(Entrant::new(3))
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
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(4))
                ]),
                Match::new([EntrantSpot::Entrant(Entrant::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(Entrant::new(2)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(Entrant::new(3)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Entrant::new(1))]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(2)),
                    EntrantSpot::Entrant(Entrant::new(3))
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
    fn test_double_elimination_iter() {
        let entrants = entrants![0, 1, 2, 3, 4];
        let tournament = DoubleElimination::<i32, u32>::new(entrants);

        let mut iter_upper = tournament.upper_bracket_iter();

        assert_eq!(
            iter_upper.next().unwrap(),
            [
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(0)),
                    EntrantSpot::Entrant(Entrant::new(4))
                ]),
                Match::new([EntrantSpot::Entrant(Entrant::new(1)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(Entrant::new(2)), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(Entrant::new(3)), EntrantSpot::Empty]),
            ]
        );
        assert_eq!(
            iter_upper.next().unwrap(),
            [
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(Entrant::new(1))]),
                Match::new([
                    EntrantSpot::Entrant(Entrant::new(2)),
                    EntrantSpot::Entrant(Entrant::new(3))
                ]),
            ]
        );
        assert_eq!(
            iter_upper.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),]
        );
        assert_eq!(iter_upper.next(), None);

        let mut iter_lower = tournament.lower_bracket_iter();

        assert_eq!(
            iter_lower.next().unwrap(),
            [
                Match::new([EntrantSpot::TBD, EntrantSpot::Empty]),
                Match::new([EntrantSpot::Empty, EntrantSpot::Empty]),
            ]
        );
        assert_eq!(
            iter_lower.next().unwrap(),
            [
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
        assert_eq!(
            iter_lower.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),]
        );
        assert_eq!(
            iter_lower.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),]
        );
        assert_eq!(iter_lower.next(), None);

        let mut iter_final = tournament.final_bracket_iter();

        assert_eq!(
            iter_final.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),]
        );
        assert_eq!(iter_final.next(), None);
    }
}
