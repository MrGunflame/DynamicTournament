//! Bracket Generator

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
    T: Entrant + Clone,
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
            if let EntrantSpot::Empty = teams[1] {
                placeholder_matches.push(matches.len());
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
            let entrant = this.get_mut(index).unwrap().entrants[0].unwrap_ref_mut();
            entrant.set_winner(true);

            let mut entrant = entrant.clone();
            entrant.set_winner(false);

            if let Some(m) = this.next_match_mut(index) {
                m.entrants[index % 2] = EntrantSpot::Entrant(entrant);
            }
        }

        debug_assert_eq!(this.matches.len(), num_matches);

        this
    }

    /// Resume the bracket from an existing Vec of matches.
    ///
    /// Note: This assumes the given Vec contains valid data. No checks are performed.
    pub fn resume(matches: Vec<Match<T>>) -> Self {
        Self {
            initial_matches: (matches.len() + 1) / 2,
            matches,
        }
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

    /// Gets the match with the given `index`, then calls `F` on it.
    /// The return value indicates the winner of a match, a value of `None` means that
    /// no winner has been determined yet (no additional operation is performed).
    /// If `index` is out-of-bounds, `F` will never execute.
    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<T>) -> Option<MatchResult<T>>,
    {
        if index >= self.matches.len() {
            return;
        }

        let m = self.get_mut(index).unwrap();

        let winner = f(m);

        if let Some(winner) = winner {
            if let Some(next_match) = self.next_match_mut(index) {
                match winner {
                    MatchResult::Entrants { winner, looser: _ } => {
                        next_match.entrants[index % 2] = EntrantSpot::Entrant(winner);
                    }
                    MatchResult::Raw { winner, loser: _ } => {
                        next_match.entrants[index % 2] = winner;
                    }
                    MatchResult::None => {
                        next_match.entrants[index % 2] = EntrantSpot::TBD;
                    }
                }
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

        counter
    }

    /// Returns the index of the match within its round based on the match index.
    pub fn match_index(&self, index: usize) -> usize {
        let mut buffer = 0;
        let mut start = self.initial_matches;
        while index >= buffer + start {
            buffer += start;
            start /= 2;
        }

        index - buffer
    }

    pub fn iter(&self) -> impl Iterator<Item = &Match<T>> {
        self.matches.iter()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MatchResult<T> {
    Entrants {
        winner: T,
        looser: T,
    },
    Raw {
        winner: EntrantSpot<T>,
        loser: EntrantSpot<T>,
    },
    None,
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
}

/// Calculates the amount of entrants in the first round.
fn calculate_wanted_inital_entrants(amount_entants: usize) -> usize {
    // Calculate the next pow(2, n) number.
    let mut start = 1;
    while start < amount_entants {
        start <<= 1;
    }

    start
}

/// Predict the amount of matches for the whole tournament.
fn predict_amount_of_matches(starting_amount: usize) -> usize {
    let mut starting_amount = calculate_wanted_inital_entrants(starting_amount);

    let mut counter = starting_amount / 2;
    while starting_amount > 1 {
        starting_amount >>= 1;
        counter += starting_amount / 2;
    }

    counter
}

/// An wrapper around an Entrant `T` with an associated score `S`.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

/// An entrant that can be used in tournaments.
pub trait Entrant: Clone {
    /// Sets the winner state of the entrant.
    fn set_winner(&mut self, winner: bool);
}

impl<T, S> Entrant for EntrantWithScore<T, S>
where
    T: Clone,
    S: Clone,
{
    fn set_winner(&mut self, winner: bool) {
        self.winner = winner;
    }
}

impl<T> From<T> for EntrantSpot<T>
where
    T: Entrant,
{
    fn from(entrant: T) -> Self {
        Self::Entrant(entrant)
    }
}

/// A double elimination tournament.
#[derive(Clone, Debug)]
pub struct DoubleElimination<T>
where
    T: Entrant,
{
    matches: Vec<Match<T>>,
    lower_bracket_index: usize,
    final_bracket_index: usize,
    initial_matches: usize,
}

impl<T> DoubleElimination<T>
where
    T: Entrant,
{
    pub fn new(entrants: Vec<T>) -> Self {
        let num_matches = {
            let mut starting_amount = calculate_wanted_inital_entrants(entrants.len());

            let mut counter = 0;
            while starting_amount > 1 {
                // Upper bracket
                counter += starting_amount / 2;

                // Lower bracket
                counter += starting_amount / 2;

                starting_amount >>= 1;
            }

            counter
        };

        let lower_bracket_index = {
            let mut counter = 0;
            let mut num = calculate_wanted_inital_entrants(entrants.len()) / 2;
            while num >= 1 {
                counter += num;
                num /= 2;
            }

            counter
        };

        let final_bracket_index = num_matches - 1;

        #[cfg(debug_assertions)]
        if entrants.len() == 8 {
            assert_eq!(num_matches, 14);
        }

        let mut matches = Vec::with_capacity(num_matches);

        let mut placeholder_matches = Vec::new();

        let mut i = 0;
        while i < entrants.len() {
            let teams = [
                EntrantSpot::new(entrants.get(i).cloned()),
                EntrantSpot::new(entrants.get(i + 1).cloned()),
            ];

            if let EntrantSpot::Empty = teams[1] {
                placeholder_matches.push(matches.len());
            }

            matches.push(Match::new(teams));

            i += 2;
        }

        i /= 2;

        let mut i2 = 0;
        while i < calculate_wanted_inital_entrants(entrants.len()) / 2 {
            let m = matches.get_mut(i2).unwrap();
            let entrant = m.entrants[1].take();

            matches.push(Match::new([entrant, EntrantSpot::Empty]));

            placeholder_matches.push(i2);
            placeholder_matches.push(i);

            i2 += 1;
            i += 1;
        }

        while i < matches.capacity() {
            matches.push(Match::new([EntrantSpot::TBD, EntrantSpot::TBD]));

            i += 1;
        }

        let mut this = Self {
            matches,
            lower_bracket_index,
            final_bracket_index,
            initial_matches: calculate_wanted_inital_entrants(entrants.len()) / 2,
        };

        for index in placeholder_matches {
            let entrant = this.get_mut(index).unwrap().entrants[0].unwrap_ref_mut();
            entrant.set_winner(true);

            let mut entrant = entrant.clone();
            entrant.set_winner(false);

            let (winner, loser) = this.next_matches_index(index);

            if let Some(winner) = winner {
                let m = this.get_mut(winner).unwrap();
                m.entrants[index % 2] = EntrantSpot::Entrant(entrant);
            }

            if let Some(loser) = loser {
                let m = this.get_mut(loser).unwrap();
                m.entrants[index % 2] = EntrantSpot::Empty;
            }
        }

        this
    }

    pub fn resume(matches: Vec<Match<T>>) -> Self {
        let initial_matches = calculate_wanted_inital_entrants(matches.len() / 4);

        Self {
            initial_matches,
            lower_bracket_index: matches.len() / 2,
            final_bracket_index: matches.len() - 1,
            matches,
        }
    }

    pub fn len(&self) -> usize {
        self.matches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<&Match<T>> {
        self.matches.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut Match<T>> {
        self.matches.get_mut(index)
    }

    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<T>) -> Option<MatchResult<T>>,
    {
        let m = match self.get_mut(index) {
            Some(m) => m,
            None => return,
        };

        let res = f(m);

        if let Some(res) = res {
            let (winner, looser) = match res {
                MatchResult::Entrants { winner, looser } => {
                    (EntrantSpot::Entrant(winner), EntrantSpot::Entrant(looser))
                }
                MatchResult::Raw { winner, loser } => (winner, loser),
                MatchResult::None => (EntrantSpot::TBD, EntrantSpot::TBD),
            };

            match index {
                // Update a match in the final bracket.
                i if i >= self.final_bracket_index => {}
                // Update a match in the lower bracket.
                i if i >= self.lower_bracket_index => {
                    if i == self.final_bracket_index - 1 {
                        let m = self.get_mut(self.final_bracket_index).unwrap();

                        // The winner of the lower bracket takes the second spot.
                        m.entrants[1] = winner;

                        return;
                    }

                    // We need to know the number of matches in the current round, and the
                    // index of the current round (see [`Self::lower_round_index`]).
                    let mut round_index = 0;
                    let mut buffer = 0;
                    let mut num_matches = self.initial_matches / 2;

                    while index - self.lower_bracket_index >= buffer + num_matches {
                        round_index += 1;
                        buffer += num_matches;

                        if round_index % 2 == 0 {
                            num_matches /= 2;
                        }
                    }

                    // The index of the match within the current round.
                    let match_index = index - buffer - self.lower_bracket_index;

                    match round_index {
                        i if i == self.final_bracket_index - 1 => {
                            let winner_index = self.final_bracket_index;

                            let m = self.get_mut(winner_index).unwrap();

                            // The winner from the lower bracket always takes the second
                            // spot.
                            m.entrants[1] = winner;
                        }
                        i if i % 2 == 0 => {
                            let winner_index = index + num_matches;

                            let m = self.get_mut(winner_index).unwrap();

                            // The winner always takes the first spot.
                            m.entrants[0] = winner;
                        }
                        _ => {
                            let winner_index =
                                index + (num_matches - match_index + match_index / 2);

                            let m = self.get_mut(winner_index).unwrap();

                            m.entrants[(index - 1) % 2] = winner;
                        }
                    }
                }
                // Update a match in the upper bracket.
                _ => {
                    let index_winner = self.initial_matches + index / 2;

                    match index {
                        i if i == self.final_bracket_index / 2 => {
                            let match_winner = self.get_mut(self.final_bracket_index).unwrap();

                            // The winner from the upper bracket takes the first spot.
                            match_winner.entrants[0] = winner;

                            let match_looser = self.get_mut(self.final_bracket_index - 1).unwrap();
                            match_looser.entrants[1] = looser;
                        }
                        // Handle the first round differently.
                        i if i < self.initial_matches => {
                            let index_looser = self.lower_bracket_index + (i / 2);

                            let match_winner = self.get_mut(index_winner).unwrap();
                            match_winner.entrants[index % 2] = winner;

                            let match_looser = self.get_mut(index_looser).unwrap();
                            match_looser.entrants[index % 2] = looser;
                        }
                        _ => {
                            // Find the index of the match in the second round of the lower
                            // bracket with the same number of matches as in the current round.
                            let mut buffer = self.initial_matches;
                            let mut num_matches = self.initial_matches / 2;
                            let mut lower_buffer = 0;
                            while index - self.upper_match_index(index) >= buffer {
                                buffer += num_matches;
                                lower_buffer += num_matches * 2;
                                num_matches /= 2;
                            }

                            let index_loser = self.lower_bracket_index
                                + lower_buffer
                                + self.upper_match_index(index)
                                - num_matches * 2;

                            let match_winner = self.get_mut(index_winner).unwrap();
                            match_winner.entrants[index % 2] = winner;

                            let match_loser = self.get_mut(index_loser).unwrap();

                            // The looser always takes the second spot.
                            match_loser.entrants[1] = looser;
                        }
                    };
                }
            }
        }
    }

    pub fn upper_round_index(&self, index: usize) -> usize {
        let mut counter = 0;
        let mut buffer = 0;
        let mut start = self.initial_matches;
        while index >= buffer + start {
            counter += 1;
            buffer += start;
            start /= 2;
        }

        counter
    }

    pub fn upper_match_index(&self, index: usize) -> usize {
        let mut buffer = 0;
        let mut start = self.initial_matches;
        while index >= buffer + start {
            buffer += start;
            start /= 2;
        }

        index - buffer
    }

    /// Returns the index of the round in with the match at `index` is located.
    pub fn lower_round_index(&self, index: usize) -> usize {
        let mut counter = 0;
        let mut buffer = 0;
        let mut num_matches_per_round = self.initial_matches / 2;

        while index - self.lower_bracket_index >= buffer + num_matches_per_round {
            counter += 1;
            buffer += num_matches_per_round;

            if counter % 2 == 0 {
                num_matches_per_round /= 2;
            }
        }

        counter
    }

    pub fn lower_match_index(&self, index: usize) -> usize {
        let mut buffer = 0;
        let mut num_matches_per_round = self.initial_matches / 2;
        let mut is_second_round = false;

        while index - self.lower_bracket_index >= buffer + num_matches_per_round {
            buffer += num_matches_per_round;

            if is_second_round {
                num_matches_per_round /= 2;
            }
            is_second_round = !is_second_round;
        }

        index - self.lower_bracket_index - buffer
    }

    pub fn next_match_upper_mut(&mut self, index: usize) -> Option<&mut Match<T>> {
        if index != self.matches.len() - 1 {
            Some(self.get_mut(self.initial_matches + index / 2).unwrap())
        } else {
            None
        }
    }

    /// Returns the index of the winner and loser match following the match `index`.
    /// The first value in the tuple contains the winner index, the second value contains
    /// the loser index. A `None` value indicates no next match.
    pub fn next_matches_index(&self, index: usize) -> (Option<usize>, Option<usize>) {
        match index {
            // Final match:
            // No followup matches for winner or loser.
            i if i >= self.final_bracket_index => (None, None),
            // Lower bracket:
            // The winner continues in the lower bracket. The loser has no followup match.
            i if i >= self.lower_bracket_index => {
                if i == self.final_bracket_index {
                    let winner = self.final_bracket_index;

                    return (Some(winner), None);
                }

                let mut round_index = 0;
                let mut buffer = 0;
                let mut num_matches = self.initial_matches / 2;
                while index - self.lower_bracket_index >= buffer + num_matches {
                    round_index += 1;
                    buffer += num_matches;

                    if round_index % 2 == 0 {
                        num_matches /= 2;
                    }
                }

                let winner = index - buffer - self.lower_bracket_index;

                let winner = match round_index {
                    i if i == self.final_bracket_index - 1 => self.final_bracket_index,
                    i if i % 2 == 0 => index + num_matches,
                    _ => index + (num_matches - winner + winner / 2),
                };

                (Some(winner), None)
            }
            // Upper bracket:
            // The winner moves into the next round in the upper bracket. The loser moves
            // into the same round in the lower bracket.
            _ => {
                let (winner, loser);

                match index {
                    // The final match in the upper bracket. Move the winner into the final
                    // bracket. Move the loser into the final lower bracket match.
                    i if i == self.final_bracket_index / 2 => {
                        winner = self.final_bracket_index;
                        loser = self.final_bracket_index - 1;
                    }
                    // The first round of matches. All matches in the lower bracket need to
                    // be filled.
                    i if i < self.initial_matches => {
                        winner = self.initial_matches + index / 2;
                        loser = self.lower_bracket_index + (i / 2);
                    }
                    _ => {
                        winner = self.initial_matches + index / 2;

                        // Find the index of the match in the second round of the lower
                        // bracket with the same number of matches as in the current round.
                        let mut buffer = self.initial_matches;
                        let mut num_matches = self.initial_matches / 2;
                        let mut lower_buffer = 0;
                        while index - self.upper_match_index(index) >= buffer {
                            buffer += num_matches;
                            lower_buffer += num_matches * 2;
                            num_matches /= 2;
                        }

                        loser =
                            self.lower_bracket_index + lower_buffer + self.upper_match_index(index)
                                - num_matches * 2;
                    }
                }

                (Some(winner), Some(loser))
            }
        }
    }

    pub fn upper_bracket_iter(&self) -> RoundsIter<'_, T> {
        RoundsIter {
            slice: &self.matches[0..self.lower_bracket_index],
            index: 0,
            next_round: self.initial_matches,
        }
    }

    pub fn lower_bracket_iter(&self) -> LowerBracketIter<'_, T> {
        LowerBracketIter {
            start: self.lower_bracket_index,
            slice: &self.matches,
            index: 0,
            num_matches: self.initial_matches / 2,
            iter_count: 0,
        }
    }

    pub fn final_bracket_iter(&self) -> FinalBracketIter<'_, T> {
        FinalBracketIter {
            start: self.final_bracket_index,
            slice: &self.matches,
            index: 0,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Match<T>> {
        self.matches.iter()
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

impl<'a, T> Iterator for LowerBracketIter<'a, T>
where
    T: std::fmt::Debug,
{
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

#[cfg(test)]
mod tests {
    use super::*;

    // Implement Entrant for i32 for testing only.
    impl Entrant for i32 {
        fn set_winner(&mut self, _winner: bool) {}
    }

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
    fn test_double_elimination() {
        // Test with a pow(2, n) number of teams.
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = DoubleElimination::new(entrants);

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
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        assert_eq!(tournament.lower_bracket_index, 7);
        assert_eq!(tournament.final_bracket_index, 13);

        // Test with a pow(2, n) - 1 number of teams.
        // The entrant not playing in the first round continues immediately.
        let entrants = vec![0, 1, 2, 3, 4, 5, 6];
        let tournament = DoubleElimination::new(entrants);

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
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        assert_eq!(tournament.lower_bracket_index, 7);
        assert_eq!(tournament.final_bracket_index, 13);

        // Test with a pow(2, n) + 1 number of teams.
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7, 8];
        let tournament = DoubleElimination::new(entrants);

        assert_eq!(
            tournament.matches,
            vec![
                // R1
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(8), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(3), EntrantSpot::Empty]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::Empty]),
                // R2
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(2)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(8), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(3), EntrantSpot::Entrant(5)]),
                // R3
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                // R4
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                // L
                Match::new([EntrantSpot::Empty, EntrantSpot::Empty]),
                Match::new([EntrantSpot::Empty, EntrantSpot::TBD]),
                Match::new([EntrantSpot::Empty, EntrantSpot::Empty]),
                Match::new([EntrantSpot::Empty, EntrantSpot::Empty]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        assert_eq!(tournament.lower_bracket_index, 15);
        assert_eq!(tournament.final_bracket_index, 29);
    }

    #[test]
    fn test_double_elimination_next_match_inex() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = DoubleElimination::new(entrants);

        //
        // UPPER BRACKET
        //

        let (winner, loser) = tournament.next_matches_index(0);
        assert_eq!(winner, Some(4));
        assert_eq!(loser, Some(7));

        let (winner, loser) = tournament.next_matches_index(1);
        assert_eq!(winner, Some(4));
        assert_eq!(loser, Some(7));

        let (winner, loser) = tournament.next_matches_index(2);
        assert_eq!(winner, Some(5));
        assert_eq!(loser, Some(8));

        let (winner, loser) = tournament.next_matches_index(3);
        assert_eq!(winner, Some(5));
        assert_eq!(loser, Some(8));

        let (winner, loser) = tournament.next_matches_index(4);
        assert_eq!(winner, Some(6));
        assert_eq!(loser, Some(9));

        let (winner, loser) = tournament.next_matches_index(5);
        assert_eq!(winner, Some(6));
        assert_eq!(loser, Some(10));

        let (winner, loser) = tournament.next_matches_index(6);
        assert_eq!(winner, Some(13));
        assert_eq!(loser, Some(12));

        //
        // LOWER BRACKET
        //

        let (winner, loser) = tournament.next_matches_index(7);
        assert_eq!(winner, Some(9));
        assert_eq!(loser, None);

        let (winner, loser) = tournament.next_matches_index(8);
        assert_eq!(winner, Some(10));
        assert_eq!(loser, None);

        let (winner, loser) = tournament.next_matches_index(9);
        assert_eq!(winner, Some(11));
        assert_eq!(loser, None);

        let (winner, loser) = tournament.next_matches_index(10);
        assert_eq!(winner, Some(11));
        assert_eq!(loser, None);

        let (winner, loser) = tournament.next_matches_index(11);
        assert_eq!(winner, Some(12));
        assert_eq!(loser, None);

        let (winner, loser) = tournament.next_matches_index(12);
        assert_eq!(winner, Some(13));
        assert_eq!(loser, None);

        //
        // FINAL BRACKET
        //

        let (winner, loser) = tournament.next_matches_index(13);
        assert_eq!(winner, None);
        assert_eq!(loser, None);
    }

    #[test]
    fn test_double_elimination_update_match() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut tournament = DoubleElimination::new(entrants);

        tournament.update_match(0, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[0].unwrap(),
                looser: m.entrants[1].unwrap(),
            })
        });
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
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(1, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[1].unwrap(),
                looser: m.entrants[0].unwrap(),
            })
        });
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
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::Entrant(2)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(2, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[0].unwrap(),
                looser: m.entrants[1].unwrap(),
            })
        });
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::Entrant(2)]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(3, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[0].unwrap(),
                looser: m.entrants[1].unwrap(),
            })
        });
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(6)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::Entrant(2)]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(7, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[0].unwrap(),
                looser: m.entrants[1].unwrap(),
            })
        });
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(6)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::Entrant(2)]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        tournament.update_match(8, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[0].unwrap(),
                looser: m.entrants[1].unwrap(),
            })
        });
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(6)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::Entrant(2)]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );

        // Last match has no followup matches, and no changes are made.
        tournament.update_match(13, |_| {
            Some(MatchResult::Entrants {
                winner: 8,
                looser: 9,
            })
        });
        assert_eq!(
            tournament.matches,
            vec![
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(6)]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::Entrant(2)]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(1), EntrantSpot::TBD]),
                Match::new([EntrantSpot::Entrant(5), EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
    }

    #[test]
    fn test_double_elimination_lower_round_index() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = DoubleElimination::new(entrants);

        assert_eq!(tournament.lower_round_index(7), 0);
        assert_eq!(tournament.lower_round_index(8), 0);
        assert_eq!(tournament.lower_round_index(9), 1);
        assert_eq!(tournament.lower_round_index(10), 1);
        assert_eq!(tournament.lower_round_index(11), 2);
        assert_eq!(tournament.lower_round_index(12), 3);
    }

    #[test]
    fn test_double_elimination_lower_match_index() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = DoubleElimination::new(entrants);

        assert_eq!(tournament.lower_match_index(7), 0);
        assert_eq!(tournament.lower_match_index(8), 1);
        assert_eq!(tournament.lower_match_index(9), 0);
        assert_eq!(tournament.lower_match_index(10), 1);
        assert_eq!(tournament.lower_match_index(11), 0);
        assert_eq!(tournament.lower_match_index(12), 0);

        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let tournament = DoubleElimination::new(entrants);

        assert_eq!(tournament.lower_match_index(15), 0);
        assert_eq!(tournament.lower_match_index(16), 1);
        assert_eq!(tournament.lower_match_index(17), 2);
        assert_eq!(tournament.lower_match_index(18), 3);
        assert_eq!(tournament.lower_match_index(19), 0);
        assert_eq!(tournament.lower_match_index(20), 1);
        assert_eq!(tournament.lower_match_index(21), 2);
        assert_eq!(tournament.lower_match_index(22), 3);
        assert_eq!(tournament.lower_match_index(23), 0);
        assert_eq!(tournament.lower_match_index(24), 1);
        assert_eq!(tournament.lower_match_index(25), 0);
        assert_eq!(tournament.lower_match_index(26), 1);
        assert_eq!(tournament.lower_match_index(27), 0);
        assert_eq!(tournament.lower_match_index(28), 0);
    }

    #[test]
    fn test_double_elimination_upper_iter() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = DoubleElimination::new(entrants);

        let mut iter = tournament.upper_bracket_iter();
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
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
        assert_eq!(
            iter.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD])]
        );
        assert_eq!(iter.next(), None);

        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let tournament = DoubleElimination::new(entrants);

        let mut iter = tournament.upper_bracket_iter();
        assert_eq!(
            iter.next().unwrap(),
            [
                Match::new([EntrantSpot::Entrant(0), EntrantSpot::Entrant(1)]),
                Match::new([EntrantSpot::Entrant(2), EntrantSpot::Entrant(3)]),
                Match::new([EntrantSpot::Entrant(4), EntrantSpot::Entrant(5)]),
                Match::new([EntrantSpot::Entrant(6), EntrantSpot::Entrant(7)]),
                Match::new([EntrantSpot::Entrant(8), EntrantSpot::Entrant(9)]),
                Match::new([EntrantSpot::Entrant(10), EntrantSpot::Entrant(11)]),
                Match::new([EntrantSpot::Entrant(12), EntrantSpot::Entrant(13)]),
                Match::new([EntrantSpot::Entrant(14), EntrantSpot::Entrant(15)]),
            ]
        );
        assert_eq!(
            iter.next().unwrap(),
            [
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
        assert_eq!(
            iter.next().unwrap(),
            [
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
            ]
        );
        assert_eq!(
            iter.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),]
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_double_elimination_lower_iter() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = DoubleElimination::new(entrants);

        let mut iter = tournament.lower_bracket_iter();
        assert_eq!(
            iter.next().unwrap(),
            [
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD]),
                Match::new([EntrantSpot::TBD, EntrantSpot::TBD])
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
        assert_eq!(
            iter.next().unwrap(),
            [Match::new([EntrantSpot::TBD, EntrantSpot::TBD])]
        );
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_single_elimination_update_match() {
        let entrants = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let mut tournament = SingleElimination::new(entrants);

        tournament.update_match(0, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[0].unwrap(),
                looser: m.entrants[1].unwrap(),
            })
        });
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
            ],
        );

        tournament.update_match(1, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[1].unwrap(),
                looser: m.entrants[0].unwrap(),
            })
        });
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

        // No change
        tournament.update_match(1, |_| None);
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

        // Undo second update_match operation.
        tournament.update_match(1, |_| Some(MatchResult::None));
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

        // 7 is out-of-bounds and doesn't update anything.
        tournament.update_match(7, |m| {
            Some(MatchResult::Entrants {
                winner: m.entrants[0].unwrap(),
                looser: m.entrants[1].unwrap(),
            })
        });
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
