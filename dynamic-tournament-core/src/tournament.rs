use std::borrow::Borrow;

use crate::options::{TournamentOptionValues, TournamentOptions};
use crate::{
    DoubleElimination, EntrantData, Entrants, Match, MatchResult, Matches, Node, Result,
    RoundRobin, SingleElimination, Swiss, System,
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
    pub fn new(kind: TournamentKind, options: TournamentOptionValues) -> Self {
        let inner = match kind {
            TournamentKind::SingleElimination => InnerTournament::SingleElimination(
                SingleElimination::new_with_options(vec![].into_iter(), options),
            ),
            TournamentKind::DoubleElimination => {
                InnerTournament::DoubleElimination(DoubleElimination::new(vec![].into_iter()))
            }
            TournamentKind::RoundRobin => {
                InnerTournament::RountRobin(RoundRobin::new(vec![].into_iter()))
            }
            TournamentKind::Swiss => InnerTournament::Swiss(Swiss::new(vec![].into_iter())),
        };

        Self { inner }
    }

    pub fn options(kind: TournamentKind) -> TournamentOptions {
        match kind {
            TournamentKind::SingleElimination => SingleElimination::<T, D>::options(),
            TournamentKind::DoubleElimination => TournamentOptions::default(),
            TournamentKind::RoundRobin => TournamentOptions::default(),
            TournamentKind::Swiss => Swiss::<T, D>::options(),
        }
    }

    pub fn resume(
        kind: TournamentKind,
        entrants: Entrants<T>,
        matches: Matches<D>,
        options: TournamentOptionValues,
    ) -> Result<Self> {
        let inner = match kind {
            TournamentKind::SingleElimination => InnerTournament::SingleElimination(
                SingleElimination::resume(entrants, matches, options)?,
            ),
            TournamentKind::DoubleElimination => {
                InnerTournament::DoubleElimination(DoubleElimination::resume(entrants, matches)?)
            }
            TournamentKind::RoundRobin => {
                InnerTournament::RountRobin(RoundRobin::resume(entrants, matches)?)
            }
            TournamentKind::Swiss => {
                InnerTournament::Swiss(Swiss::resume(entrants, matches, options)?)
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
            InnerTournament::RountRobin(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.push(entrant);
                *t = RoundRobin::new(entrants.entrants.into_iter());
            }
            InnerTournament::Swiss(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.push(entrant);
                *t = Swiss::new(entrants.entrants.into_iter());
            }
        }
    }

    pub fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<D>>, &mut MatchResult<D>),
    {
        match &mut self.inner {
            InnerTournament::SingleElimination(t) => t.update_match(index, f),
            InnerTournament::DoubleElimination(t) => t.update_match(index, f),
            InnerTournament::RountRobin(t) => t.update_match(index, f),
            InnerTournament::Swiss(t) => t.update_match(index, f),
        }
    }
}

impl<T, D> Extend<T> for Tournament<T, D>
where
    T: Clone,
    D: EntrantData + Clone,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        match &mut self.inner {
            InnerTournament::SingleElimination(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.extend(iter);
                *t = SingleElimination::new(entrants.entrants.into_iter());
            }
            InnerTournament::DoubleElimination(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.extend(iter);
                *t = DoubleElimination::new(entrants.entrants.into_iter());
            }
            InnerTournament::RountRobin(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.extend(iter);
                *t = RoundRobin::new(entrants.entrants.into_iter());
            }
            InnerTournament::Swiss(t) => {
                let mut entrants = t.clone().into_entrants();
                entrants.extend(iter);
                *t = Swiss::new(entrants.entrants.into_iter());
            }
        }
    }
}

impl<T, D> System for Tournament<T, D>
where
    T: Clone,
    D: EntrantData + Clone,
{
    type Entrant = T;
    type NodeData = D;

    fn entrants(&self) -> &Entrants<Self::Entrant> {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.entrants(),
            InnerTournament::DoubleElimination(t) => t.entrants(),
            InnerTournament::RountRobin(t) => t.entrants(),
            InnerTournament::Swiss(t) => t.entrants(),
        }
    }

    unsafe fn entrants_mut(&mut self) -> &mut Entrants<Self::Entrant> {
        unsafe {
            match &mut self.inner {
                InnerTournament::SingleElimination(t) => t.entrants_mut(),
                InnerTournament::DoubleElimination(t) => t.entrants_mut(),
                InnerTournament::RountRobin(t) => t.entrants_mut(),
                InnerTournament::Swiss(t) => t.entrants_mut(),
            }
        }
    }

    fn into_entrants(self) -> Entrants<Self::Entrant> {
        match self.inner {
            InnerTournament::SingleElimination(t) => t.into_entrants(),
            InnerTournament::DoubleElimination(t) => t.into_entrants(),
            InnerTournament::RountRobin(t) => t.into_entrants(),
            InnerTournament::Swiss(t) => t.into_entrants(),
        }
    }

    fn matches(&self) -> &Matches<Self::NodeData> {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.matches(),
            InnerTournament::DoubleElimination(t) => t.matches(),
            InnerTournament::RountRobin(t) => t.matches(),
            InnerTournament::Swiss(t) => t.matches(),
        }
    }

    unsafe fn matches_mut(&mut self) -> &mut Matches<Self::NodeData> {
        unsafe {
            match &mut self.inner {
                InnerTournament::SingleElimination(t) => t.matches_mut(),
                InnerTournament::DoubleElimination(t) => t.matches_mut(),
                InnerTournament::RountRobin(t) => t.matches_mut(),
                InnerTournament::Swiss(t) => t.matches_mut(),
            }
        }
    }

    fn into_matches(self) -> Matches<Self::NodeData> {
        match self.inner {
            InnerTournament::SingleElimination(t) => t.into_matches(),
            InnerTournament::DoubleElimination(t) => t.into_matches(),
            InnerTournament::RountRobin(t) => t.into_matches(),
            InnerTournament::Swiss(t) => t.into_matches(),
        }
    }

    fn next_matches(&self, index: usize) -> crate::NextMatches {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.next_matches(index),
            InnerTournament::DoubleElimination(t) => t.next_matches(index),
            InnerTournament::RountRobin(t) => t.next_matches(index),
            InnerTournament::Swiss(t) => t.next_matches(index),
        }
    }

    fn update_match<F>(&mut self, index: usize, f: F)
    where
        F: FnOnce(&mut Match<Node<Self::NodeData>>, &mut MatchResult<Self::NodeData>),
    {
        match &mut self.inner {
            InnerTournament::SingleElimination(t) => t.update_match(index, f),
            InnerTournament::DoubleElimination(t) => t.update_match(index, f),
            InnerTournament::RountRobin(t) => t.update_match(index, f),
            InnerTournament::Swiss(t) => t.update_match(index, f),
        }
    }

    fn start_render(&self) -> crate::render::RenderState<'_, Self> {
        // Transmute the returned `RenderState<T>` into `RenderState<Self>`. This is safe since a
        // `RenderState` only contains a reference to the system and all operations will go through
        // `self`.
        match &self.inner {
            InnerTournament::SingleElimination(t) => unsafe {
                std::mem::transmute(t.start_render())
            },
            InnerTournament::DoubleElimination(t) => unsafe {
                std::mem::transmute(t.start_render())
            },
            InnerTournament::RountRobin(t) => unsafe { std::mem::transmute(t.start_render()) },
            InnerTournament::Swiss(t) => unsafe { std::mem::transmute(t.start_render()) },
        }
    }

    fn standings(&self) -> crate::standings::Standings {
        match &self.inner {
            InnerTournament::SingleElimination(t) => t.standings(),
            InnerTournament::DoubleElimination(t) => t.standings(),
            InnerTournament::RountRobin(t) => t.standings(),
            InnerTournament::Swiss(t) => t.standings(),
        }
    }
}

impl<T, D> Borrow<Entrants<T>> for Tournament<T, D>
where
    T: Clone,
    D: EntrantData + Clone,
{
    fn borrow(&self) -> &Entrants<T> {
        self.entrants()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TournamentKind {
    SingleElimination,
    DoubleElimination,
    RoundRobin,
    Swiss,
}

#[derive(Clone, Debug)]
enum InnerTournament<T, D>
where
    D: EntrantData,
{
    SingleElimination(SingleElimination<T, D>),
    DoubleElimination(DoubleElimination<T, D>),
    RountRobin(RoundRobin<T, D>),
    Swiss(Swiss<T, D>),
}
