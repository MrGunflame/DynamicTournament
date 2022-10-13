//! # Tournament Rendering
//!
//! The `render` module provides types to generically render tournament [`System`]s.
//!
//! The rendering process is built around three components which can be used to build any
//! tournament tree:
//! - A [`Column`] is a repeating vertical container element.
//! - A [`Row`] is a repeating horizontal container element.
//! - A [`Match`] is a leaf element displaying match at a specific index.
use crate::System;

use std::{marker::PhantomData, ops::Deref};

/// A renderer used to render any [`System`].
pub trait Renderer<T, E, D>
where
    T: System<Entrant = E, NodeData = D>,
{
    fn render(&mut self, root: Container<'_, T>);
}

/// A wrapper around a list of elements.
///
/// A `Container` can be thought of as a node in a AST representing the tournament tree.
#[derive(Debug)]
pub struct Container<'a, T>
where
    T: System,
{
    pub(crate) inner: ContainerInner<'a, T>,
    pub(crate) position: Position,
}

impl<'a, T> Container<'a, T>
where
    T: System,
{
    pub fn position(&self) -> Position {
        self.position
    }

    /// Returns the [`ElementKind`] of the elements this `Container` wraps around.
    pub fn kind(&self) -> ElementKind {
        match self.inner {
            ContainerInner::Columns(_) => ElementKind::Column,
            ContainerInner::Rows(_) => ElementKind::Row,
            ContainerInner::Matches(_) => ElementKind::Match,
        }
    }

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

/// A leaf element in the render tree representing a *match* or *heat*.
#[derive(Clone, Debug)]
pub struct Match<'a, T>
where
    T: System,
{
    pub(crate) index: usize,
    pub(crate) predecessors: Vec<Predecessor>,
    pub(crate) position: Option<Position>,
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
    /// A `None` value indicates that the `Match` should be rendered using the hint from the
    /// parent.
    #[inline]
    pub fn position(&self) -> Option<Position> {
        self.position
    }

    /// Returns a non-exhaustive list of [`Predecessor`]s leading to this `Match`. All elements are
    /// guaranteed to be correct, but there is no guarantee that the list is exhaustive.
    pub fn predecessors(&self) -> &[Predecessor] {
        &self.predecessors
    }
}

/// A predecessor hint of a match.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Predecessor {
    pub kind: PredecessorKind,
    /// The index of the match that is the predecessor.
    pub source_match: usize,
    /// Destination index within the next match.
    pub destination_index: usize,
    pub(crate) _priv: (),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PredecessorKind {
    Winner,
    Loser,
}

/// A `Position` gives the renderer a hint how the [`System`] expects this element to be displayed.
///
/// Note that a `Position` is purely a hint, a renderer may decide to ignore it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Position {
    /// Hints that the element should be rendered at the start of the container.
    ///
    /// # Examples
    ///
    /// ```
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
    /// ```
    Start,
    /// Hints that the element should be rendered at the end of the container.
    ///
    /// # Examples
    ///
    /// ```
    /// |     COL0     |     COL1     |     COL2     |
    /// | ------------ | ------------ | ------------ |
    /// |              |              |              |
    /// | | -------- | |              |              |
    /// | | Match[0] | |              |              |
    /// | | -------- | |              |              |
    /// |              |              |              |
    /// | | -------- | |              |              |
    /// | | Match[1] | |              |              |
    /// | | -------- | |              |              |
    /// |              |              |              |
    /// | | -------- | | | -------- | |              |
    /// | | Match[2] | | | Match[4] | |              |
    /// | | -------- | | | -------- | |              |
    /// |              |              |              |
    /// | | -------- | | | -------- | | | -------- | |
    /// | | Match[3] | | | Match[5] | | | Match[6] | |
    /// | | -------- | | | -------- | | | -------- | |
    /// |              |              |              |
    /// ```
    End,
    SpaceAround,
    SpaceBetween,
}

/// An `Iterator` over a list of [`Column`]s with a defined length.
#[derive(Debug)]
pub struct ColumnsIter<'a, T>
where
    T: System,
{
    slice: &'a [Column<'a, T>],
}

impl<'a, T> Iterator for ColumnsIter<'a, T>
where
    T: System,
{
    type Item = &'a Column<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (elem, rem) = self.slice.split_first()?;
        self.slice = rem;
        Some(elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a, T> ExactSizeIterator for ColumnsIter<'a, T>
where
    T: System,
{
    fn len(&self) -> usize {
        self.slice.len()
    }
}

#[derive(Debug)]
pub struct RowsIter<'a, T>
where
    T: System,
{
    slice: &'a [Row<'a, T>],
}

impl<'a, T> Iterator for RowsIter<'a, T>
where
    T: System,
{
    type Item = &'a Row<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (elem, rem) = self.slice.split_first()?;
        self.slice = rem;
        Some(elem)
    }
}

#[derive(Debug)]
pub struct MatchesIter<'a, T>
where
    T: System,
{
    slice: &'a [Match<'a, T>],
}

impl<'a, T> Iterator for MatchesIter<'a, T>
where
    T: System,
{
    type Item = &'a Match<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (elem, rem) = self.slice.split_first()?;
        self.slice = rem;
        Some(elem)
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
        match &inner.inner {
            ContainerInner::Columns(cols) => Self::Columns(ColumnsIter { slice: cols }),
            ContainerInner::Rows(rows) => Self::Rows(RowsIter { slice: rows }),
            ContainerInner::Matches(matches) => Self::Matches(MatchesIter { slice: matches }),
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

/// The type of an element.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ElementKind {
    Column,
    Row,
    Match,
}

impl ElementKind {
    /// Returns `true` if this `ElementKind` is [`Column`].
    ///
    /// [`Column`]: Self::Column
    #[inline]
    pub fn is_column(&self) -> bool {
        matches!(self, Self::Column)
    }

    /// Returns `true` if this `ElementKind` is [`Row`].
    ///
    /// [`Row`]: Self::Row
    #[inline]
    pub fn is_row(&self) -> bool {
        matches!(self, Self::Row)
    }

    /// Returns `true` if this `ElementKind` is [`Match`].
    ///
    /// [`Match`]: Self::Match
    #[inline]
    pub fn is_match(&self) -> bool {
        matches!(self, Self::Match)
    }
}
