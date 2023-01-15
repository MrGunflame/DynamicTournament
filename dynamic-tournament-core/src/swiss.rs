use std::borrow::Borrow;
use std::cmp::Ordering;

use crate::render::{Column, Element, Position, RenderState, Row};
use crate::utils::NumExt;
use crate::{
    EntrantData, EntrantSpot, Entrants, Match, MatchResult, Matches, NextMatches, Node, System,
};

// Implementation based on the Monrad sytem:
// The inital round is based on each opponent played against the next, i.e. #1 v #2, #3 v #4, etc
// For all other rounds the entrants are sorted based on their score with first priority, and their
// initial position with second priority.
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
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        let entrants: Entrants<T> = entrants.collect();

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
        while index < entrants.len() {
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

        Self {
            matches_done_vec: vec![false; matches.len()],
            matches_done: 0,
            entrants,
            matches,
            scores,
            options: SwissOptions::default(),
        }
    }

    fn build_next_round(&mut self) {
        if self.matches_done % self.matches_per_round() != 0 {
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

    fn next_matches(&self, index: usize) -> NextMatches {
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

            if let Some((index, _)) = res.winner {
                if let EntrantSpot::Entrant(index) = index {
                    let cell = self
                        .scores
                        .iter_mut()
                        .find(|cell| cell.index == index)
                        .unwrap();

                    cell.score += self.options.score_win;
                }
            }

            if let Some((index, _)) = res.loser {
                if let EntrantSpot::Entrant(index) = index {
                    let cell = self
                        .scores
                        .iter_mut()
                        .find(|cell| cell.index == index)
                        .unwrap();

                    cell.score += self.options.score_loss;
                }
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
    use crate::{entrants, EntrantSpot, Match, Node, System};

    use super::{Cell, Swiss};

    #[test]
    fn test_swiss() {
        let entrants = entrants![];
        let tournament = Swiss::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, []);
        assert_eq!(tournament.matches, []);

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
}
