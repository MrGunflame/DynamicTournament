use crate::{EntrantData, EntrantSpot, Entrants, Match, Matches, Node, System};

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
                let first = if first <= entrants.len() - 1 {
                    EntrantSpot::Entrant(Node::new(first))
                } else {
                    EntrantSpot::Empty
                };

                let second = if second <= entrants.len() - 1 {
                    EntrantSpot::Entrant(Node::new(second))
                } else {
                    EntrantSpot::Empty
                };

                matches.push(Match::new([first, second]));
            }
        }

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
            res if res <= 0 => n - (res.abs() as usize) - 1,
            res => {
                debug_assert!(res > 0);

                res as usize
            }
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
}

#[cfg(test)]
mod tests {
    use crate::{entrants, EntrantSpot, Match, Node};

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
}
