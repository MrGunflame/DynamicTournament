//! # Tournament Rendering
//!
//! The `render` module provides types to generically render tournament [`System`]s.
use crate::System;

use std::{marker::PhantomData, ops::Deref};

/// A renderer used to render any [`System`].
pub trait Renderer<T, E, D>
where
    T: System<Entrant = E, NodeData = D>,
{
    fn render(&mut self, root: Container<'_, T>);
}

#[derive(Debug)]
pub struct Container<'a, T>
where
    T: System,
{
    pub(crate) inner: ContainerInner<'a, T>,
}

impl<'a, T> Container<'a, T>
where
    T: System,
{
    pub fn iter(&self) -> ContainerIter<'_, T> {
        ContainerIter::new(self)
    }
}

#[derive(Debug)]
pub(crate) enum ContainerInner<'a, T>
where
    T: System,
{
    Columns(Vec<Column<'a, T>>),
    Rows(Vec<Row<'a, T>>),
    Matches(Vec<Match<'a, T>>),
}

#[derive(Debug)]
pub struct Column<'a, T>
where
    T: System,
{
    pub(crate) inner: Container<'a, T>,
}

impl<'a, T> Deref for Column<'a, T>
where
    T: System,
{
    type Target = Container<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct Row<'a, T>
where
    T: System,
{
    pub inner: Container<'a, T>,
}

impl<'a, T> Deref for Row<'a, T>
where
    T: System,
{
    type Target = Container<'a, T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Clone, Debug)]
pub struct Match<'a, T>
where
    T: System,
{
    pub(crate) index: usize,
    pub(crate) predecessors: Vec<usize>,
    pub(crate) position: Position,
    pub(crate) _marker: PhantomData<&'a T>,
}

impl<'a, T> Match<'a, T>
where
    T: System,
{
    /// Returns the index of this `Match` within the [`System`].
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Returns the hinted [`Position`] at which the [`System`] expectes the match to be rendered.
    #[inline]
    pub fn position(&self) -> Position {
        self.position
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Predecessor {
    source_match: usize,
    /// Destination index within the next match.
    destination_index: usize,
}

/// A `Position` gives the renderer a hint where the [`System`] expects this match to be displayed.
///
/// Note that a `Position` is purely a hint, a renderer may decide to ignore it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    /// Hints that the match should be rendered at the start of the container.
    ///
    /// # Examples
    ///
    /// |     COL0     |     COL1     |     COL2     |
    /// | ------------ | ------------ | ------------ |
    /// |              |              |              |
    /// | | -------- | | | -------- | | | -------- | |
    /// | | Match[0] | | | Match[4] | | | Match[6] | |
    /// | | -------- | | | -------- | | | -------- | |
    /// |              |              |              |
    /// | | -------- | | | -------- | |              |
    /// | | Match[1] | | | Match[5] | |              |
    /// | | -------- | | | -------- | |              |
    /// |              |              |              |
    /// | | -------- | |              |              |
    /// | | Match[2] | |              |              |
    /// | | -------- | |              |              |
    /// |              |              |              |
    /// | | -------- | |              |              |
    /// | | Match[3] | |              |              |
    /// | | -------- | |              |              |
    /// |              |              |              |
    Start,
    End,
    SpaceAround,
    SpaceBetween,
}

#[derive(Debug)]
pub struct ColumnsIter<'a, T>
where
    T: System,
{
    inner: &'a ContainerInner<'a, T>,
    pos: usize,
}

impl<'a, T> ColumnsIter<'a, T>
where
    T: System,
{
    unsafe fn new(inner: &'a Container<'a, T>) -> Self {
        Self {
            inner: &inner.inner,
            pos: 0,
        }
    }
}

impl<'a, T> Iterator for ColumnsIter<'a, T>
where
    T: System,
{
    type Item = &'a Column<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            ContainerInner::Columns(vec) => {
                let val = vec.get(self.pos)?;
                self.pos += 1;
                Some(val)
            }
            _ => {
                if cfg!(debug_assertions) {
                    unreachable!()
                } else {
                    unsafe { std::hint::unreachable_unchecked() }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct RowsIter<'a, T>
where
    T: System,
{
    inner: &'a ContainerInner<'a, T>,
    pos: usize,
}

impl<'a, T> RowsIter<'a, T>
where
    T: System,
{
    unsafe fn new(inner: &'a Container<'a, T>) -> Self {
        Self {
            inner: &inner.inner,
            pos: 0,
        }
    }
}

impl<'a, T> Iterator for RowsIter<'a, T>
where
    T: System,
{
    type Item = &'a Row<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            ContainerInner::Rows(vec) => {
                let val = vec.get(self.pos)?;
                self.pos += 1;
                Some(val)
            }
            _ => {
                if cfg!(debug_assertions) {
                    unreachable!()
                } else {
                    unsafe { std::hint::unreachable_unchecked() }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct MatchesIter<'a, T>
where
    T: System,
{
    inner: &'a ContainerInner<'a, T>,
    pos: usize,
}

impl<'a, T> MatchesIter<'a, T>
where
    T: System,
{
    unsafe fn new(inner: &'a Container<'a, T>) -> Self {
        Self {
            inner: &inner.inner,
            pos: 0,
        }
    }
}

impl<'a, T> Iterator for MatchesIter<'a, T>
where
    T: System,
{
    type Item = &'a Match<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            ContainerInner::Matches(vec) => {
                let val = vec.get(self.pos)?;
                self.pos += 1;
                Some(val)
            }
            _ => {
                if cfg!(debug_assertions) {
                    unreachable!()
                } else {
                    unsafe { std::hint::unreachable_unchecked() }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum ContainerIter<'a, T>
where
    T: System,
{
    Columns(ColumnsIter<'a, T>),
    Rows(RowsIter<'a, T>),
    Matches(MatchesIter<'a, T>),
}

impl<'a, T> ContainerIter<'a, T>
where
    T: System,
{
    fn new(inner: &'a Container<'a, T>) -> Self {
        unsafe {
            match &inner.inner {
                ContainerInner::Columns(_) => Self::Columns(ColumnsIter::new(inner)),
                ContainerInner::Rows(_) => Self::Rows(RowsIter::new(inner)),
                ContainerInner::Matches(_) => Self::Matches(MatchesIter::new(inner)),
            }
        }
    }
}

#[derive(Debug)]
pub struct RenderState<'a, T>
where
    T: System,
{
    pub(crate) inner: Container<'a, T>,
}
