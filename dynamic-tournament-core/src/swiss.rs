use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::options::{TournamentOptionValues, TournamentOptions};
use crate::render::{Column, Element, Position, RenderState, Row};
use crate::standings::Standings;
use crate::utils::NumExt;
use crate::{
    EntrantData, EntrantSpot, Entrants, Error, Match, MatchResult, Matches, NextMatches, Node,
    Result, System,
};

/// A swiss group stage tournament.
///
/// # Implementation notes
///
/// The current pairing system is based on the Monrad system, the current tie-breaking system is
/// based on the Buchholz system.
///
/// Note that the concrete implementation might change in the future.
// Implementation based on the Monrad sytem:
// The inital round is based on each opponent played against the next, i.e. #1 v #2, #3 v #4, etc
// For all other rounds the entrants are sorted based on their score with first priority, and their
// initial position with second priority.
// If the number of entrants is odd, we only have entrants - 1 matches per round and the last
// entrant (lowest score/starting position) is excluded for the round. They then receive a point
// to prevent being excluded in the next round again.
// Tie-breaking is based on the Buchholz system:
#[derive(Clone, Debug)]
pub struct Swiss<T, D> {
    entrants: Entrants<T>,
    matches: Matches<D>,
    scores: Vec<Cell>,
    options: SwissOptions,
    matches_done: usize,
    // FIXME: Remove this vec and get the information elsewhere (or use a different format).
    matches_done_vec: Vec<bool>,
}

impl<T, D> Swiss<T, D>
where
    D: EntrantData + Default,
{
    /// Creates a new `Swiss` tournament using the given `entrants`.
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        Self::new_with_options(entrants, TournamentOptionValues::default())
    }

    /// Creates a new `Swiss` tournament using the given `entrants` and using the given `options`.
    ///
    /// If you don't need to specify the options consider using [`new`].
    ///
    /// [`new`]: Self::new
    pub fn new_with_options<I, O>(entrants: I, options: O) -> Self
    where
        I: Iterator<Item = T>,
        O: Into<TournamentOptionValues>,
    {
        let entrants: Entrants<T> = entrants.collect();
        let options = SwissOptions::new(options.into());

        let num_rounds = match entrants.len() {
            0 => 0,
            n => n.ilog2_ceil(),
        };

        let num_matches = match entrants.len() % 2 {
            0 => entrants.len(),
            _ => entrants.len() - 1,
        } / 2;

        let mut matches = Matches::with_capacity(num_rounds * num_matches);

        // Build the first round.
        let mut index = 0;
        for _ in 0..num_matches {
            let first = EntrantSpot::Entrant(Node::new(index));
            let second = EntrantSpot::Entrant(Node::new(index + 1));

            matches.push(Match::new([first, second]));

            index += 2;
        }

        // Remaining rounds.
        if num_rounds > 0 {
            for _ in 0..(num_rounds - 1) * num_matches {
                matches.push(Match::tbd());
            }
        }

        let mut scores = Vec::with_capacity(entrants.len());
        for index in 0..entrants.len() {
            scores.push(Cell {
                index,
                score: 0,
                initial_position: index,
            });
        }

        // Odd number of entrants, the excluded entrant gets a point.
        if entrants.len() % 2 != 0 {
            scores.last_mut().unwrap().score += options.score_bye;
        }

        Self {
            matches_done_vec: vec![false; matches.len()],
            matches_done: 0,
            entrants,
            matches,
            scores,
            options,
        }
    }

    /// Returns the [`TournamentOptions`] accepted by this system.
    pub fn options() -> TournamentOptions {
        TournamentOptions::builder()
            .option("score_win", "How many points to award for a win.", 1u64)
            .option("score_loss", "How many points to award for a loss.", 0u64)
            .option("score_bye", "How many points to award for a bye.", 1u64)
            .build()
    }

    /// Resumes the bracket from existing matches.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if `matches` has an invalid number of matches for `entrants` or
    /// an [`Node`] in `matches` points to a value that is out-of-bounds.
    pub fn resume<O>(entrants: Entrants<T>, matches: Matches<D>, options: O) -> Result<Self>
    where
        O: Into<TournamentOptionValues>,
    {
        let options = options.into();

        let num_rounds = match entrants.len() {
            0 => 0,
            n => n.ilog2_ceil(),
        };

        let num_matches = match entrants.len() % 2 {
            0 => entrants.len(),
            _ => entrants.len() - 1,
        } / 2;

        let expected = num_matches * num_rounds;

        let found = matches.len();

        if found != expected {
            return Err(Error::InvalidNumberOfMatches { expected, found });
        }

        for match_ in matches.iter() {
            for entrant in match_.entrants.iter() {
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

        unsafe { Ok(Self::resume_unchecked(entrants, matches, options)) }
    }

    /// Resumes the bracket from existing matches without validating `matches`.
    ///
    /// # Safety
    ///
    /// Calling this function with a number of `matches` that is not valid for the length of
    /// `entrants` or points to a entrant that is out-of-bounds is undefined behaivoir.
    pub unsafe fn resume_unchecked<O>(
        entrants: Entrants<T>,
        matches: Matches<D>,
        options: O,
    ) -> Self
    where
        O: Into<TournamentOptionValues>,
    {
        let options = SwissOptions::new(options.into());

        // Rebuild scores.
        let mut scores = Vec::with_capacity(entrants.len());
        for index in 0..entrants.len() {
            scores.push(Cell {
                index,
                score: 0,
                initial_position: index,
            });
        }

        let mut matches_done_vec = vec![false; matches.len()];

        for (i, match_) in matches.iter().enumerate() {
            for spot in &match_.entrants {
                if let EntrantSpot::Entrant(node) = spot {
                    if node.data.winner() {
                        let cell = scores
                            .iter_mut()
                            .find(|cell| cell.index == node.index)
                            .unwrap();

                        cell.score += 1;

                        matches_done_vec[i] = true;
                    }
                }
            }
        }

        Self {
            entrants,
            matches,
            options,
            matches_done: matches_done_vec.iter().filter(|b| **b).count(),
            matches_done_vec,
            scores,
        }
    }

    fn build_next_round(&mut self) {
        if self.matches_done % self.matches_per_round() != 0 {
            return;
        }

        // Tournament done.
        if self.matches_done == self.matches().len() {
            return;
        }

        self.scores.sort();

        let round = self.matches_done / self.matches_per_round();

        // FIXME: Get rid of this clone.
        let entrants = self.scores.clone();
        let mut index = 0;
        for match_ in self.round_mut(round) {
            let first = entrants[index].index;
            let second = entrants[index + 1].index;

            *match_ = Match::new([
                EntrantSpot::Entrant(Node::new(first)),
                EntrantSpot::Entrant(Node::new(second)),
            ]);

            index += 2;
        }

        // Odd number of entrants, the excluded entrant gets a point.
        if self.scores.len() % 2 != 0 {
            self.scores.last_mut().unwrap().score += self.options.score_bye;
        }
    }

    fn round(&self, round: usize) -> &[Match<Node<D>>] {
        let start = self.matches_per_round() * round;
        let end = start + self.matches_per_round();

        self.matches.get(start..end).unwrap()
    }

    fn round_mut(&mut self, round: usize) -> &mut [Match<Node<D>>] {
        let start = self.matches_per_round() * round;
        let end = start + self.matches_per_round();

        self.matches.get_mut(start..end).unwrap()
    }

    fn matches_per_round(&self) -> usize {
        (match self.entrants.len() % 2 {
            0 => self.entrants.len(),
            _ => self.entrants.len() - 1,
        }) / 2
    }
}

#[derive(Copy, Clone, Debug)]
struct SwissOptions {
    score_win: usize,
    score_loss: usize,
    score_bye: usize,
}

impl SwissOptions {
    pub fn new(mut options: TournamentOptionValues) -> Self {
        let mut this = Self::default();

        if let Some(val) = options.take("score_win") {
            this.score_win = val.unwrap_u64_or(1) as usize;
        }

        if let Some(val) = options.take("score_loss") {
            this.score_loss = val.unwrap_u64_or(0) as usize;
        }

        if let Some(val) = options.take("score_bye") {
            this.score_bye = val.unwrap_u64_or(1) as usize;
        }

        this
    }
}

impl Default for SwissOptions {
    fn default() -> Self {
        Self {
            score_win: 1,
            score_loss: 0,
            score_bye: 1,
        }
    }
}

impl<T, D> System for Swiss<T, D>
where
    D: EntrantData,
{
    type Entrant = T;
    type NodeData = D;

    #[inline]
    fn entrants(&self) -> &Entrants<Self::Entrant> {
        &self.entrants
    }

    #[inline]
    unsafe fn entrants_mut(&mut self) -> &mut Entrants<Self::Entrant> {
        &mut self.entrants
    }

    #[inline]
    fn into_entrants(self) -> Entrants<Self::Entrant> {
        self.entrants
    }

    #[inline]
    fn matches(&self) -> &Matches<Self::NodeData> {
        &self.matches
    }

    #[inline]
    unsafe fn matches_mut(&mut self) -> &mut Matches<Self::NodeData> {
        &mut self.matches
    }

    #[inline]
    fn into_matches(self) -> Matches<Self::NodeData> {
        self.matches
    }

    fn next_matches(&self, _index: usize) -> NextMatches {
        NextMatches::default()
    }

    fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<Self::NodeData>>, &mut MatchResult<Self::NodeData>),
    {
        let Some(match_) = self.matches.get_mut(index) else {
            return;
        };

        let mut res = MatchResult::default();
        f(match_, &mut res);

        if res.reset {
            unimplemented!();
        }

        if !self.matches_done_vec[index] {
            self.matches_done_vec[index] = true;
            self.matches_done += 1;

            if let Some((EntrantSpot::Entrant(index), _)) = res.winner {
                let cell = self
                    .scores
                    .iter_mut()
                    .find(|cell| cell.index == index)
                    .unwrap();

                cell.score += self.options.score_win;
            }

            if let Some((EntrantSpot::Entrant(index), _)) = res.loser {
                let cell = self
                    .scores
                    .iter_mut()
                    .find(|cell| cell.index == index)
                    .unwrap();

                cell.score += self.options.score_loss;
            }
        }

        self.build_next_round();
    }

    fn start_render(&self) -> RenderState<'_, Self> {
        let mut rounds = Vec::new();

        let matches_per_round = match self.entrants.len() % 2 {
            0 => self.entrants.len(),
            _ => self.entrants.len() - 1,
        } / 2;

        let mut index = 0;
        while index < self.matches.len() {
            let mut round = Vec::new();

            for _ in 0..matches_per_round {
                round.push(Element::new(crate::render::Match {
                    index,
                    predecessors: vec![],
                    _marker: std::marker::PhantomData,
                    label: None,
                    position: None,
                }));

                index += 1;
            }

            rounds.push(Element::new(Row {
                label: None,
                position: Some(Position::Start),
                children: round.into_iter(),
            }));
        }

        RenderState {
            root: Element::new(Column::new(rounds)),
        }
    }

    fn standings(&self) -> Standings {
        #[derive(Clone, Debug, PartialEq, Eq, Default)]
        struct Score {
            wins: u64,
            loses: u64,
            byes: u64,
            score: u64,
            buchholz: u64,
            /// The opponents that this entrant has played against.
            opponents: Vec<usize>,
        }

        impl PartialOrd for Score {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for Score {
            fn cmp(&self, other: &Self) -> Ordering {
                self.score
                    .cmp(&other.score)
                    .then_with(|| self.buchholz.cmp(&other.buchholz))
                    .reverse()
            }
        }

        let mut scores = HashMap::new();
        // Make sure every entrant has an entry.
        for index in 0..self.entrants().len() {
            if !scores.contains_key(&index) {
                scores.insert(index, Score::default());
            }
        }

        let rounds = self.matches.len() / self.matches_per_round();
        for round in 0..rounds {
            let round = self.round(round);

            let mut round_entrants = HashSet::new();
            for index in 0..self.entrants.len() {
                round_entrants.insert(index);
            }

            for match_ in round {
                // Skip matches that are not complete.
                if !match_.is_concluded() {
                    continue;
                }

                for (i, entrant) in match_.entrants.iter().enumerate() {
                    let node = entrant.unwrap_ref();

                    round_entrants.remove(&node.index);
                    let mut score = scores.get_mut(&node.index).unwrap();

                    if node.data.winner() {
                        score.wins += 1;
                    } else {
                        score.loses += 1;
                    }

                    let opponent = match i {
                        0 => match_.entrants[1].unwrap_ref().index,
                        _ => match_.entrants[0].unwrap_ref().index,
                    };

                    score.opponents.push(opponent);
                }
            }

            // One entrant may have a bye.
            if let Some(index) = round_entrants.into_iter().next() {
                scores.get_mut(&index).unwrap().byes += 1;
            }
        }

        // Clone scores to avoid some borrowing things.
        // FIXME: Remove this clone, which is not necessary as an entrant
        // can never have itself as an opponent. (e.g. with UnsafeCell)
        let scores2 = scores.clone();
        for cell in &self.scores {
            let mut scores = scores.get_mut(&cell.index).unwrap();
            scores.score += cell.score as u64;

            // Calculate the Median-Buchholz rating.
            // The "raw" score is equivalent to the number of wins. Draws
            // are not considered in the current system.
            let mut buchholz = 0;
            let mut highest_score = 0;
            let mut lowest_score = None;
            for opponent in &scores.opponents {
                let raw_score = scores2.get(opponent).unwrap().wins;

                if raw_score > highest_score {
                    buchholz += highest_score;
                    highest_score = raw_score;
                } else if raw_score < lowest_score.unwrap_or(0) {
                    if let Some(lowest_score) = lowest_score {
                        buchholz += lowest_score;
                    }

                    lowest_score = Some(raw_score);
                } else {
                    buchholz += raw_score;
                }
            }

            scores.buchholz = buchholz;
        }

        // Sort the entries by wins and losses (reversed).
        let mut entries: Vec<_> = scores.into_iter().collect();
        entries.sort_by(|(_, a), (_, b)| a.cmp(b));

        let mut builder = Standings::builder();
        builder.key("Wins");
        builder.key("Losses");
        builder.key("Byes");
        builder.key("Score");
        builder.key("Buchholz");

        for (index, score) in entries {
            builder.entry(index, |builder| {
                builder.value(score.wins);
                builder.value(score.loses);
                builder.value(score.byes);
                builder.value(score.score);
                builder.value(score.buchholz);
            });
        }

        builder.build()
    }
}

impl<T, D> Borrow<Entrants<T>> for Swiss<T, D> {
    #[inline]
    fn borrow(&self) -> &Entrants<T> {
        &self.entrants
    }
}

/// A cell with an entrant.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Cell {
    /// The index of the entrant.
    index: usize,
    score: usize,
    initial_position: usize,
}

impl PartialOrd for Cell {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cell {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        match self.score.cmp(&other.score) {
            Ordering::Less => Ordering::Greater,
            Ordering::Equal => self.initial_position.cmp(&other.initial_position),
            Ordering::Greater => Ordering::Less,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::options::TournamentOptionValues;
    use crate::tests::{TColumn, TElement, TMatch, TRow, TestRenderer};
    use crate::{
        entrants, EntrantScore, EntrantSpot, Entrants, Error, Match, Matches, Node, System,
    };

    use super::{Cell, Swiss};

    #[test]
    fn test_swiss() {
        let entrants = entrants![];
        let tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, []);
        assert_eq!(tournament.matches, []);

        let entrants = entrants![0];
        let tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0]);
        assert_eq!(tournament.matches, []);

        let entrants = entrants![0, 1];
        let tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1]);
        assert_eq!(
            tournament.matches,
            [Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(1)),
            ])]
        );

        let entrants = entrants![0, 1, 2];
        let tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2]);
        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::tbd(),
            ]
        );

        let entrants = entrants![0, 1, 2, 3];
        let tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2, 3]);
        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::tbd(),
                Match::tbd(),
            ]
        );

        let entrants = entrants![0, 1, 2, 3, 4, 5, 6, 7];
        let tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(
            tournament.matches,
            vec![
                // Round 0
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(6)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 1
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                // Round 2
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
            ]
        );
        assert_eq!(
            tournament.scores,
            vec![
                Cell {
                    index: 0,
                    initial_position: 0,
                    score: 0,
                },
                Cell {
                    index: 1,
                    initial_position: 1,
                    score: 0,
                },
                Cell {
                    index: 2,
                    initial_position: 2,
                    score: 0,
                },
                Cell {
                    index: 3,
                    initial_position: 3,
                    score: 0
                },
                Cell {
                    index: 4,
                    initial_position: 4,
                    score: 0,
                },
                Cell {
                    index: 5,
                    initial_position: 5,
                    score: 0,
                },
                Cell {
                    index: 6,
                    initial_position: 6,
                    score: 0,
                },
                Cell {
                    index: 7,
                    initial_position: 7,
                    score: 0
                }
            ]
        );
    }

    #[test]
    fn test_swiss_update_match() {
        let entrants = entrants![0, 1, 2, 3, 4, 5, 6, 7];
        let mut tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(
            tournament.matches,
            vec![
                // Round 0
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(6)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 1
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                // Round 2
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
            ]
        );

        // No effect until all matches of the round ended.
        tournament.update_match(0, |m, res| {
            res.winner_default(&m[0]);
        });

        assert_eq!(
            tournament.matches,
            vec![
                // Round 0
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(6)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 1
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                // Round 2
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
            ]
        );

        for index in 0..4 {
            tournament.update_match(index, |m, res| {
                res.winner_default(&m[0]);
                res.loser_default(&m[1]);
            });
        }

        assert_eq!(
            tournament.matches,
            vec![
                // Round 0
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(6)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 1
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(5)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 2
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
            ]
        );

        for index in 4..8 {
            tournament.update_match(index, |m, res| {
                res.winner_default(&m[0]);
                res.loser_default(&m[1]);
            });
        }

        assert_eq!(
            tournament.matches,
            vec![
                // Round 0
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(6)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 1
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(5)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 2
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(4)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(5)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(3)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
            ]
        );
    }

    #[test]
    fn test_swiss_resume() {
        let entrants = Entrants::from(vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let matches = Matches::from(vec![
            // Round 0
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(1)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(2)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(4)),
                EntrantSpot::Entrant(Node::new(5)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(6)),
                EntrantSpot::Entrant(Node::new(7)),
            ]),
            // Round 1
            Match::tbd(),
            Match::tbd(),
            Match::tbd(),
            Match::tbd(),
            // Round 2
            Match::tbd(),
            Match::tbd(),
            Match::tbd(),
            Match::tbd(),
        ]);

        let mut tournament =
            Swiss::<i32, u32>::resume(entrants, matches, TournamentOptionValues::default())
                .unwrap();

        for index in 0..4 {
            tournament.update_match(index, |m, res| {
                res.winner_default(&m[0]);
                res.loser_default(&m[1]);
            });
        }

        assert_eq!(
            tournament.matches,
            vec![
                // Round 0
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(6)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 1
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(5)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 2
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
                Match::tbd(),
            ]
        );

        for index in 4..8 {
            tournament.update_match(index, |m, res| {
                res.winner_default(&m[0]);
                res.loser_default(&m[1]);
            });
        }

        assert_eq!(
            tournament.matches,
            vec![
                // Round 0
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(5)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(6)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 1
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(4)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(5)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
                // Round 2
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(4)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(5)),
                    EntrantSpot::Entrant(Node::new(6)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(3)),
                    EntrantSpot::Entrant(Node::new(7)),
                ]),
            ]
        );
    }

    #[test]
    fn test_swiss_resume_score() {
        let entrants = Entrants::from(vec![0, 1, 2, 3, 4, 5, 6, 7]);
        let matches = Matches::from(vec![
            // Round 0
            Match::new([
                EntrantSpot::Entrant(Node::new_with_data(
                    0,
                    EntrantScore {
                        score: 1,
                        winner: true,
                    },
                )),
                EntrantSpot::Entrant(Node::new(1)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new_with_data(
                    2,
                    EntrantScore {
                        score: 1,
                        winner: true,
                    },
                )),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new_with_data(
                    4,
                    EntrantScore {
                        score: 1,
                        winner: true,
                    },
                )),
                EntrantSpot::Entrant(Node::new(5)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new_with_data(
                    6,
                    EntrantScore {
                        score: 1,
                        winner: true,
                    },
                )),
                EntrantSpot::Entrant(Node::new(7)),
            ]),
            // Round 1
            Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Entrant(Node::new(2)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(4)),
                EntrantSpot::Entrant(Node::new(6)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(1)),
                EntrantSpot::Entrant(Node::new(3)),
            ]),
            Match::new([
                EntrantSpot::Entrant(Node::new(5)),
                EntrantSpot::Entrant(Node::new(7)),
            ]),
            // Round 2
            Match::tbd(),
            Match::tbd(),
            Match::tbd(),
            Match::tbd(),
        ]);
        let options = TournamentOptionValues::default();

        let tournament =
            Swiss::<u32, EntrantScore<u32>>::resume(entrants, matches, options).unwrap();

        assert_eq!(
            tournament.scores,
            [
                Cell {
                    index: 0,
                    initial_position: 0,
                    score: 1
                },
                Cell {
                    index: 1,
                    initial_position: 1,
                    score: 0
                },
                Cell {
                    index: 2,
                    initial_position: 2,
                    score: 1,
                },
                Cell {
                    index: 3,
                    initial_position: 3,
                    score: 0
                },
                Cell {
                    index: 4,
                    initial_position: 4,
                    score: 1
                },
                Cell {
                    index: 5,
                    initial_position: 5,
                    score: 0
                },
                Cell {
                    index: 6,
                    initial_position: 6,
                    score: 1
                },
                Cell {
                    index: 7,
                    initial_position: 7,
                    score: 0
                }
            ]
        );
    }

    #[test]
    fn test_swiss_resume_invalid() {
        let entrants = Entrants::from(vec![0, 1, 2, 3]);
        let matches = Matches::new();
        let options = TournamentOptionValues::default();

        assert_eq!(
            Swiss::<i32, u32>::resume(entrants, matches, options).unwrap_err(),
            Error::InvalidNumberOfMatches {
                expected: 4,
                found: 0,
            }
        );

        let entrants = Entrants::from(vec![0, 1]);
        let matches = Matches::from(vec![Match::new([
            EntrantSpot::Entrant(Node::new(0)),
            EntrantSpot::Entrant(Node::new(2)),
        ])]);
        let options = TournamentOptionValues::default();

        assert_eq!(
            Swiss::<i32, u32>::resume(entrants, matches, options).unwrap_err(),
            Error::InvalidEntrant {
                index: 2,
                length: 2
            }
        );
    }

    #[test]
    fn test_swiss_render() {
        let entrants = entrants![0, 1, 2, 3];
        let tournament = Swiss::<i32, u32>::new(entrants);

        let mut renderer = TestRenderer::new();
        tournament.render(&mut renderer);

        assert_eq!(
            renderer,
            TElement::Column(TColumn(vec![
                TElement::Row(TRow(vec![
                    TElement::Match(TMatch { index: 0 }),
                    TElement::Match(TMatch { index: 1 }),
                ])),
                TElement::Row(TRow(vec![
                    TElement::Match(TMatch { index: 2 }),
                    TElement::Match(TMatch { index: 3 }),
                ])),
            ]))
        );
    }
}
