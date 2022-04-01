//! Bracket Generator

/// A single elimination tournament.
#[derive(Clone, Debug)]
pub struct SingleElimination<T> {
    matches: Vec<Match<T>>,
    /// The number of initial matches (round 0).
    initial_matches: usize,
}

// FIMXE: Remove T: Clone trait bound
impl<T> SingleElimination<T>
where
    T: Clone,
{
    /// Creates a new `SingleElimination` tournament.
    // FIXME: Replace Vec<T> with a better suited type.
    pub fn new(entrants: Vec<T>) -> Self {
        let num_matches = predict_amount_of_matches(entrants.len());
        let mut matches = Vec::with_capacity(num_matches);

        // Placeholder matches are matches with only a single entrant. This is required to make
        // the bracket even. Entrants of placeholder matches will advance the next round
        // immediatly.
        // FIXME: Can pre allocate this.
        let mut placeholder_matches = Vec::new();

        let mut i = 0;
        while i < entrants.len() {
            let teams = [
                EntrantSpot::new(entrants.get(i).cloned()),
                EntrantSpot::new(entrants.get(i + 1).cloned()),
            ];

            // Mark matches with only a single team in as placeholder matches.
            // Only the second entrant can be `EntrantSpot::Empty`.
            // FIXME: Unnecessary double comparison here and above.
            match teams[1] {
                EntrantSpot::Empty => placeholder_matches.push(matches.len()),
                _ => (),
            }

            matches.push(Match::new(teams));

            // Go to next row of 2 entrants.
            i += 2;
        }

        // i / 2 is the amount of matches currently (first round).
        i /= 2;

        // If entrants.len() is a at least pow(2, n) - 2 we will have completely empty
        // matches in the first round. To fix that, we will move teams from full matches
        // until every match has at least 1 team.
        let mut i2 = 0;
        while i < calculate_wanted_inital_entrants(entrants.len()) / 2 {
            let m = matches.get_mut(i2).unwrap();
            let entrant = m.entrants[1].take();

            matches.push(Match::new([entrant, EntrantSpot::Empty]));

            // All new matches and matches where a team was removed are now placeholder
            // matches.
            placeholder_matches.push(i2);
            placeholder_matches.push(i);

            i2 += 1;
            i += 1;
        }

        // Fill `matches` will TBD matches.
        while i < matches.capacity() {
            matches.push(Match::new([EntrantSpot::TBD, EntrantSpot::TBD]));

            i += 1;
        }

        let mut this = Self {
            matches,
            initial_matches: calculate_wanted_inital_entrants(entrants.len()) / 2,
        };

        // Move all placeholder matches to the second round.
        for index in placeholder_matches {
            // FIXME: Remove unncessary clone.
            let entrant = this.get(index).unwrap().entrants[0].clone();

            if let Some(m) = this.next_match_mut(index) {
                m.entrants[index % 2] = entrant;
            }
        }

        debug_assert_eq!(this.matches.len(), num_matches);

        this
    }

    /// Returns the match with the given index.
    pub fn get(&self, index: usize) -> Option<&Match<T>> {
        if index < self.matches.len() {
            // SAFETY: index is within the bounds
            unsafe { Some(self.get_unchecked(index)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference to the match with the given index.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Match<T>> {
        if index < self.matches.len() {
            // SAFETY: index is within the bounds
            unsafe { Some(self.get_unchecked_mut(index)) }
        } else {
            None
        }
    }

    /// Returns the match with the given index.
    ///
    /// # Safety
    ///
    /// Calling this method on an index that is out of bounds causes unidentified behavoir.
    pub unsafe fn get_unchecked(&self, index: usize) -> &Match<T> {
        self.matches.get_unchecked(index)
    }

    /// Returns the [`Match`] at `index` without checking the bounds.
    ///
    /// # Safety
    ///
    /// Calling this method with an `index` that is out-of-bounds is undefined behavoir.
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut Match<T> {
        self.matches.get_unchecked_mut(index)
    }

    /// Returns the next match that the winner of the match with the given index will play.
    /// Returns `None` if there is no next game.
    ///
    /// # Panics
    ///
    /// Panics when the given index is out of bounds.
    pub fn next_match_mut(&mut self, index: usize) -> Option<&mut Match<T>> {
        if index != self.matches.len() - 1 {
            Some(self.get_mut(self.initial_matches + index / 2).unwrap())
        } else {
            None
        }
    }

    /// Returns an iterator over all rounds.
    pub fn rounds_iter(&self) -> RoundsIter<'_, T> {
        RoundsIter {
            slice: &self.matches,
            index: 0,
            next_round: self.initial_matches,
        }
    }

    /// Update the winner of a match, moving the bracket state forward.
    pub fn update_winner(&mut self, index: usize, winner: Winner) {
        self.update_winner_callback(index, winner, |_| {});
    }

    /// Update a winner, calling `f` ON THE NEXT MATCH.
    pub fn update_winner_callback<F>(&mut self, index: usize, winner: Winner, f: F)
    where
        F: FnOnce(&mut Match<T>),
    {
        // index is out of bounds.
        if index >= self.matches.len() {
            return;
        }

        match winner {
            Winner::Team(i) => {
                let entrant = self.get(index).unwrap().entrants[i].clone();

                // Get the next match, or return if there's no next match.
                let m = match self.next_match_mut(index) {
                    Some(m) => m,
                    None => return,
                };

                m.entrants[index % 2] = entrant;

                f(m);
            }
            Winner::None => {
                // Get the next match, or return if there's no next match.
                let m = match self.next_match_mut(index) {
                    Some(m) => m,
                    None => return,
                };

                m.entrants[index % 2] = EntrantSpot::TBD;
            }
        }
    }

    /// Returns the index of the round the match is located in based on the
    /// match index.
    pub fn round_index(&self, index: usize) -> usize {
        let mut counter = 0;
        let mut buffer = 0;
        let mut start = self.initial_matches;
        while index >= buffer + start {
            counter += 1;
            buffer += start;
            start /= 2;
        }

        return counter;
    }

    /// Returns the index of the match within its round based on the match index.
    pub fn match_index(&self, index: usize) -> usize {
        let mut buffer = 0;
        let mut start = self.initial_matches;
        while index >= buffer + start {
            buffer += start;
            start /= 2;
        }

        let counter = index - buffer;

        return counter;
    }
}

/// A winner for a [`Match`]. This is only usefull in [`SingleElimination::update_winner`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Winner {
    Team(usize),
    None,
}

/// A match consisting of at 2 parties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Match<T> {
    pub entrants: [EntrantSpot<T>; 2],
}

impl<T> Match<T> {
    pub fn new(entrants: [EntrantSpot<T>; 2]) -> Self {
        Self { entrants }
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

    /// Takes out an the value, leaving [`Self::Empty`] in its place.
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Self::Empty)
    }

    /// Unwraps `self` value, panicking if it is not [`Self::Entrant`].
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

    /// Unwraps `self` value, panicking if it is not [`Self::Entrant`].
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
}

/// Calculates the amount of entrants in the first round.
fn calculate_wanted_inital_entrants(amount_entants: usize) -> usize {
    // Calculate the next pow(2, n) number.
    let mut start = 1;
    while start < amount_entants {
        start = start << 1;
    }

    start
}

/// Predict the amount of matches for the whole tournament.
fn predict_amount_of_matches(starting_amount: usize) -> usize {
    let mut starting_amount = calculate_wanted_inital_entrants(starting_amount);

    let mut counter = starting_amount / 2;
    while starting_amount > 1 {
        starting_amount = starting_amount >> 1;
        counter += starting_amount / 2;
    }

    counter
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct EntrantWithScore<T, S> {
    pub entrant: T,
    pub score: S,
    pub winner: bool,
}

impl<T, S> EntrantWithScore<T, S>
where
    S: Default,
{
    /// Creates a new `EntrantWithScore` with a score of 0.
    pub fn new(entrant: T) -> Self {
        EntrantWithScore {
            entrant,
            score: S::default(),
            winner: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_amount_of_matches() {
        let entrants = 8;
        assert_eq!(predict_amount_of_matches(entrants), 7);

        let entrants = 2;
        assert_eq!(predict_amount_of_matches(entrants), 1);

        let entrants = 7;
        assert_eq!(predict_amount_of_matches(entrants), 7);
    }

    #[test]
    fn test_single_elimination_update_match() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut tournament = SingleElimination::new(entrants);

        tournament.update_winner(0, Winner::Team(0));
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_winner(1, Winner::Team(1));
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        // Unset the previous winner.
        tournament.update_winner(1, Winner::None);
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        // Index 7 is out of bounds and shouldn't update anything.
        tournament.update_winner(7, Winner::Team(1));
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
    }

    #[test]
    fn test_single_elimination_rounds_iter() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = SingleElimination::new(entrants);

        let mut iter = tournament.rounds_iter();
        assert_eq!(
            iter.next().unwrap(),
            [
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
            ]
        );
        assert_eq!(
            iter.next().unwrap(),
            [
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD])
            ]
        );
        assert_eq!(
            iter.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD])]
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_single_elimination() {
        // Test with a pow(2, n) number of teams.
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = SingleElimination::new(entrants);

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        // Test with a pow(2, n) - 1 number of teams.
        // The entrant not playing in the first round continues immediately.
        let entrants = vec![0, 1, 2, 3, 4, 5, 6];
        let tournament = SingleElimination::new(entrants);

        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Entrant(6)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
    }

    #[test]
    pub fn test_single_elimination_round_index() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = SingleElimination::new(entrants);

        assert_eq!(tournament.round_index(0), 0);
        assert_eq!(tournament.round_index(1), 0);
        assert_eq!(tournament.round_index(2), 0);
        assert_eq!(tournament.round_index(3), 0);
        assert_eq!(tournament.round_index(4), 1);
        assert_eq!(tournament.round_index(5), 1);
        assert_eq!(tournament.round_index(6), 2);
    }

    #[test]
    pub fn test_single_elimination_match_index() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = SingleElimination::new(entrants);

        assert_eq!(tournament.match_index(0), 0);
        assert_eq!(tournament.match_index(1), 1);
        assert_eq!(tournament.match_index(2), 2);
        assert_eq!(tournament.match_index(3), 3);
        assert_eq!(tournament.match_index(4), 0);
        assert_eq!(tournament.match_index(5), 1);
        assert_eq!(tournament.match_index(6), 0);
    }
}
