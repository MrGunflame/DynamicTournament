use std::borrow::Borrow;

use crate::render::{Column, Element, Position, RenderState, Row};
use crate::{
    EntrantData, EntrantSpot, Entrants, Error, Match, MatchResult, Matches, NextMatches, Node,
    Result, System,
};

#[derive(Clone, Debug)]
pub struct RoundRobin<T, D>
where
    D: EntrantData,
{
    entrants: Entrants<T>,
    matches: Matches<D>,
}

impl<T, D> RoundRobin<T, D>
where
    D: EntrantData,
{
    pub fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = T>,
    {
        let entrants: Entrants<T> = entrants.collect();

        log::debug!(
            "Creating new RoundRobin bracket with {} entrants",
            entrants.len()
        );

        let num_rounds = match entrants.len() {
            0 => 0,
            1 => 1,
            n => n - 1,
        };

        // entrants.len() if even, entrants.len() +1 if odd.
        let entrants_even = if entrants.len() % 2 == 0 {
            entrants.len()
        } else {
            entrants.len() + 1
        };

        let matches_per_round = match entrants_even {
            0 => 0,
            n => n / 2,
        };

        let mut matches = Matches::with_capacity(num_rounds * matches_per_round);

        dbg!(num_rounds);
        dbg!(matches_per_round);

        // Start by creating two rows: 0..=n/2 and n/2+1..=n.
        // Pin entrant 0 to match 0 for every round.
        // For every round rotate the upper row once to the right,
        // placing the entrant at n/2 at n (second row). Rotate the lower row
        // once to the left, placing the entrant at n/2+1 at 1.
        for round in 0..num_rounds {
            // The first round with special handling for odd entrants.
            // let first = EntrantSpot::Entrant(Node::new(0));
            // let second = if entrants.len() % 2 == 0 {
            //     EntrantSpot::Entrant(Node::new(entrants.len() - 1))
            // } else {
            //     EntrantSpot::Empty
            // };

            // matches.push(Match::new([first, second]));

            // The rest of the round.
            for index in 0..matches_per_round {
                // Take an entrant from the high and low row.
                let first = Self::circle_entrant(entrants_even, round, index);
                let second = Self::circle_entrant(entrants_even, round, entrants_even - index - 1);

                // TODO: These if cases should best not be in this hot loop.
                let first = if first < entrants.len() {
                    EntrantSpot::Entrant(Node::new(first))
                } else {
                    EntrantSpot::Empty
                };

                let second = if second < entrants.len() {
                    EntrantSpot::Entrant(Node::new(second))
                } else {
                    EntrantSpot::Empty
                };

                matches.push(Match::new([first, second]));
            }
        }

        Self { entrants, matches }
    }

    pub fn resume(entrants: Entrants<T>, matches: Matches<D>) -> Result<Self> {
        log::debug!(
            "Trying to resume RoundRobin bracket with {} entrants and {} matches",
            entrants.len(),
            matches.len()
        );

        let expected = match entrants.len() {
            0 => 0,
            n => (n / 2) * (n - 1),
        };

        if matches.len() != expected {
            return Err(Error::InvalidNumberOfMatches {
                expected,
                found: matches.len(),
            });
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

        unsafe { Ok(Self::resume_unchecked(entrants, matches)) }
    }

    #[inline]
    pub unsafe fn resume_unchecked(entrants: Entrants<T>, matches: Matches<D>) -> Self {
        log::debug!(
            "Resuming RoundRobin bracket with {} entrants and {} matches",
            entrants.len(),
            matches.len()
        );

        Self { entrants, matches }
    }

    /// Returns the output index of the circle given the input `index` of a tournament with
    /// `n` entrants at `round`.
    // #[inline]
    // fn circle_index(n: usize, round: usize, index: usize) -> usize {
    //     // 0 is pinned.
    //     if index == 0 {
    //         return 0;
    //     }

    //     match index + round {
    //         // The max, wrap around to 1.
    //         i if i >= n => (i % n) + 1,
    //         i => i,
    //     }
    // }

    /// Returns the index of entrant of the at the given `index` in a circle of length `n` at
    /// the given `round`.
    #[inline]
    fn circle_entrant(n: usize, round: usize, index: usize) -> usize {
        debug_assert!(n % 2 == 0);

        if index == 0 {
            return 0;
        }

        match index as isize - round as isize {
            res if res <= 0 => n - res.unsigned_abs() - 1,
            res => {
                debug_assert!(res > 0);

                res as usize
            }
        }
    }

    #[inline]
    fn entrants_even(&self) -> usize {
        let len = self.entrants.len();

        if len % 2 == 0 {
            len
        } else {
            len + 1
        }
    }
}

impl<T, D> System for RoundRobin<T, D>
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

    fn next_matches(&self, _: usize) -> NextMatches {
        unimplemented!()
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
    }

    fn start_render(&self) -> RenderState<'_, Self> {
        let mut rounds = Vec::new();

        let matches_per_round = self.entrants_even() / 2;

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

impl<T, D> Borrow<Entrants<T>> for RoundRobin<T, D>
where
    D: EntrantData,
{
    #[inline]
    fn borrow(&self) -> &Entrants<T> {
        &self.entrants
    }
}

impl<T, D> Borrow<Matches<D>> for RoundRobin<T, D>
where
    D: EntrantData,
{
    #[inline]
    fn borrow(&self) -> &Matches<D> {
        &self.matches
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{TElement, TMatch, TRow, TestRenderer};
    use crate::{entrants, EntrantSpot, Match, Node, System};

    use super::RoundRobin;

    // #[test]
    // fn test_circle_index() {
    //     let entrants = 14;
    //     let mut round = 0;

    //     macro_rules! test {
    //         ($($in:expr => $out:expr),*,) => {
    //             $(
    //                 assert_eq!(RoundRobin::<(), u32>::circle_index(entrants, round, $in), $out);
    //             )*
    //         };
    //     }

    //     test! {
    //         0 => 0,
    //         1 => 1,
    //         2 => 2,
    //         3 => 3,
    //         4 => 4,
    //         5 => 5,
    //         6 => 6,
    //         7 => 7,
    //         8 => 8,
    //         9 => 9,
    //         10 => 10,
    //         11 => 11,
    //         12 => 12,
    //         13 => 13,
    //     }

    //     round = 1;

    //     test! {
    //         0 => 0,
    //         1 => 2,
    //         2 => 3,
    //         3 => 4,
    //         4 => 5,
    //         5 => 6,
    //         6 => 7,
    //         7 => 8,
    //         8 => 9,
    //         9 => 10,
    //         10 => 11,
    //         11 => 12,
    //         12 => 13,
    //         13 => 1,
    //     }
    // }

    #[test]
    fn test_circle_entrant() {
        let entrants = 10;
        let mut round = 0;

        macro_rules! test {
            ($($in:expr => $out:expr),*,) => {
                $(
                    assert_eq!(RoundRobin::<(), u32>::circle_entrant(entrants, round, $in), $out);
                )*
            };
        }

        test! {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 5,
            6 => 6,
            7 => 7,
            8 => 8,
            9 => 9,
        }

        round = 1;

        test! {
            0 => 0,
            1 => 9,
            2 => 1,
            3 => 2,
            4 => 3,
            5 => 4,
            6 => 5,
            7 => 6,
            8 => 7,
            9 => 8,
        }

        round = 2;

        test! {
            0 => 0,
            1 => 8,
            2 => 9,
            3 => 1,
            4 => 2,
            5 => 3,
            6 => 4,
            7 => 5,
            8 => 6,
            9 => 7,
        }
    }

    #[test]
    fn test_round_robin() {
        let entrants = entrants![];
        let tournament = RoundRobin::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, []);
        assert_eq!(tournament.matches, []);

        let entrants = entrants![0];
        let tournament = RoundRobin::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0]);
        assert_eq!(
            tournament.matches,
            [Match::new([
                EntrantSpot::Entrant(Node::new(0)),
                EntrantSpot::Empty,
            ])]
        );

        let entrants = entrants![0, 1, 2];
        let tournament = RoundRobin::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2]);
        assert_eq!(
            tournament.matches,
            [
                Match::new([EntrantSpot::Entrant(Node::new(0)), EntrantSpot::Empty,]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([EntrantSpot::Empty, EntrantSpot::Entrant(Node::new(1))])
            ]
        );

        let entrants = entrants![0, 1, 2, 3];
        let tournament = RoundRobin::<i32, u32>::new(entrants);

        assert_eq!(tournament.entrants, [0, 1, 2, 3]);
        assert_eq!(
            tournament.matches,
            [
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(1)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(2)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(3)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(0)),
                    EntrantSpot::Entrant(Node::new(1)),
                ]),
                Match::new([
                    EntrantSpot::Entrant(Node::new(2)),
                    EntrantSpot::Entrant(Node::new(3)),
                ]),
            ]
        );
    }

    #[test]
    fn test_round_robin_render() {
        let entrants = entrants![0, 1, 2, 3];
        let tournament = RoundRobin::<i32, u32>::new(entrants);

        let mut renderer = TestRenderer::new();
        tournament.render(&mut renderer);

        assert_eq!(
            renderer,
            TElement::Row(TRow(vec![
                TElement::Row(TRow(vec![
                    TElement::Match(TMatch { index: 0 }),
                    TElement::Match(TMatch { index: 1 }),
                ])),
                TElement::Row(TRow(vec![
                    TElement::Match(TMatch { index: 2 }),
                    TElement::Match(TMatch { index: 3 }),
                ])),
                TElement::Row(TRow(vec![
                    TElement::Match(TMatch { index: 4 }),
                    TElement::Match(TMatch { index: 5 }),
                ])),
            ]))
        );
    }
}
