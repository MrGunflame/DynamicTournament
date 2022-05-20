use std::borrow::Borrow;

use crate::{
    render::{BracketRounds, Renderer},
    DoubleElimination, Entrant, EntrantData, EntrantRefMut, Entrants, Match, MatchResult, Matches,
    Result, SingleElimination,
};

#[derive(Clone, Debug)]
pub struct Tournament<T, D>
where
    T: Clone,
    D: EntrantData + Clone,
{
    inner: InnerTournament<T, D>,
}

impl<T, D> Tournament<T, D>
where
    T: Clone,
    D: EntrantData + Clone,
{
    pub fn new(kind: TournamentKind) -> Self {
        let inner = match kind {
            TournamentKind::SingleElimination => {
                InnerTournament::SingleElimination(SingleElimination::new(vec![].into_iter()))
            }
            TournamentKind::DoubleElimination => {
                InnerTournament::DoubleElimination(DoubleElimination::new(vec![].into_iter()))
            }
        };

        Self { inner }
    }

    pub fn resume(
        kind: TournamentKind,
        entrants: Entrants<T>,
        matches: Matches<Entrant<D>>,
    ) -> Result<Self> {
        let inner = match kind {
            TournamentKind::SingleElimination => {
                InnerTournament::SingleElimination(SingleElimination::resume(entrants, matches)?)
            }
            TournamentKind::DoubleElimination => {
                InnerTournament::DoubleElimination(DoubleElimination::resume(entrants, matches)?)
            }
        };

        Ok(Self { inner })
    }

    pub fn push(&mut self, entrant: T) {
        match &mut self.inner {
            InnerTournament::SingleElimination(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.push(entrant);
                *t = SingleElimination::new(entrants.entrants.into_iter());
            }
            InnerTournament::DoubleElimination(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.push(entrant);
                *t = DoubleElimination::new(entrants.entrants.into_iter());
            }
        }
    }

    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<EntrantRefMut<'_, T, D>>, &mut MatchResult<D>),
    {
        match &mut self.inner {
            InnerTournament::SingleElimination(t) => t.update_match(index, f),
            InnerTournament::DoubleElimination(t) => t.update_match(index, f),
        }
    }

    pub fn render<R>(&self, renderer: &mut R)
    where
        R: Renderer<Self, T, D>,
    {
        renderer.render(BracketRounds::new(self));
    }
}

impl<T, D> crate::Tournament for Tournament<T, D>
where
    T: Clone,
    D: EntrantData + Clone,
{
    type Entrant = T;
    type NodeData = D;

    fn new<I>(entrants: I) -> Self
    where
        I: Iterator<Item = Self::Entrant>,
    {
        unimplemented!()
    }

    fn resume(
        entrants: Entrants<Self::Entrant>,
        matches: Matches<Entrant<Self::NodeData>>,
    ) -> Result<Self> {
        unimplemented!()
    }

    unsafe fn resume_unchecked(
        entrants: Entrants<Self::Entrant>,
        matches: Matches<Entrant<Self::NodeData>>,
    ) -> Self {
        unimplemented!()
    }

    unsafe fn entrants_mut(&mut self) -> &mut Entrants<Self::Entrant> {
        unimplemented!()
    }

    fn into_entrants(self) -> Entrants<Self::Entrant> {
        unimplemented!()
    }

    fn matches(&self) -> &Matches<Entrant<Self::NodeData>> {
        unimplemented!()
    }

    unsafe fn matches_mut(&mut self) -> &mut Matches<Entrant<Self::NodeData>> {
        unimplemented!()
    }

    fn into_matches(self) -> Matches<Entrant<Self::NodeData>> {
        unimplemented!()
    }

    fn next_bracket_round(&self, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.next_bracket_round(range),
            InnerTournament::DoubleElimination(t) => t.next_bracket_round(range),
        }
    }

    fn next_bracket(&self, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.next_bracket(range),
            InnerTournament::DoubleElimination(t) => t.next_bracket(range),
        }
    }

    fn next_round(&self, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.next_round(range),
            InnerTournament::DoubleElimination(t) => t.next_round(range),
        }
    }

    fn next_matches(&self, index: usize) -> crate::NextMatches {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.next_matches(index),
            InnerTournament::DoubleElimination(t) => t.next_matches(index),
        }
    }

    fn entrants(&self) -> &Entrants<Self::Entrant> {
        unimplemented!()
    }

    fn render<R>(&self, renderer: &mut R)
    where
        R: Renderer<Self, Self::Entrant, Self::NodeData>,
    {
        unimplemented!()
    }

    fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(
            &mut Match<EntrantRefMut<'_, Self::Entrant, Self::NodeData>>,
            &mut MatchResult<Self::NodeData>,
        ),
    {
        unimplemented!()
    }
}

impl<T, D> Borrow<Entrants<T>> for Tournament<T, D>
where
    T: Clone,
    D: EntrantData + Clone,
{
    fn borrow(&self) -> &Entrants<T> {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TournamentKind {
    SingleElimination,
    DoubleElimination,
}

#[derive(Clone, Debug)]
enum InnerTournament<T, D>
where
    D: EntrantData,
{
    SingleElimination(SingleElimination<T, D>),
    DoubleElimination(DoubleElimination<T, D>),
}
