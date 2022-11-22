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

use std::borrow::Cow;
use std::marker::PhantomData;
use std::vec::IntoIter;

/// A renderer used to render any [`System`].
pub trait Renderer<T, E, D>
where
    T: System<Entrant = E, NodeData = D>,
{
    fn render(&mut self, root: Element<'_, T>);
}

#[derive(Debug)]
pub struct Element<'a, T>
where
    T: System,
{
    pub label: Option<Cow<'a, str>>,
    pub position: Option<Position>,
    pub inner: ElementInner<'a, T>,
}

impl<'a, T> Element<'a, T>
where
    T: System,
{
    pub(crate) fn new<E>(inner: E) -> Self
    where
        E: Into<ElementInner<'a, T>>,
    {
        Self {
            label: None,
            position: None,
            inner: inner.into(),
        }
    }
}

#[derive(Debug)]
pub enum ElementInner<'a, T>
where
    T: System,
{
    Container(Container<'a, T>),
    Row(Row<'a, T>),
    Column(Column<'a, T>),
    Match(Match<'a, T>),
}

impl<'a, T> From<Container<'a, T>> for ElementInner<'a, T>
where
    T: System,
{
    fn from(b: Container<'a, T>) -> Self {
        Self::Container(b)
    }
}

impl<'a, T> From<Row<'a, T>> for ElementInner<'a, T>
where
    T: System,
{
    fn from(row: Row<'a, T>) -> Self {
        Self::Row(row)
    }
}

impl<'a, T> From<Column<'a, T>> for ElementInner<'a, T>
where
    T: System,
{
    fn from(col: Column<'a, T>) -> Self {
        Self::Column(col)
    }
}

impl<'a, T> From<Match<'a, T>> for ElementInner<'a, T>
where
    T: System,
{
    fn from(m: Match<'a, T>) -> Self {
        Self::Match(m)
    }
}

/// A direct wrapper around another [`Element`].
///
/// `Container` purely exists to wrap another [`Element`] while providing additional information.
#[derive(Debug)]
pub struct Container<'a, T>
where
    T: System,
{
    children: Box<Element<'a, T>>,
}

impl<'a, T> Container<'a, T>
where
    T: System,
{
    pub fn new(children: Element<'a, T>) -> Self {
        Self {
            children: Box::new(children),
        }
    }

    pub fn into_inner(self) -> Box<Element<'a, T>> {
        self.children
    }
}

impl<'a, T> AsRef<Element<'a, T>> for Container<'a, T>
where
    T: System,
{
    fn as_ref(&self) -> &Element<'a, T> {
        &self.children
    }
}

#[derive(Debug)]
pub struct Row<'a, T>
where
    T: System,
{
    children: IntoIter<Element<'a, T>>,
}

impl<'a, T> Row<'a, T>
where
    T: System,
{
    pub(crate) fn new(children: Vec<Element<'a, T>>) -> Self {
        Self {
            children: children.into_iter(),
        }
    }
}

impl<'a, T> Iterator for Row<'a, T>
where
    T: System,
{
    type Item = Element<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.children.next()
    }
}

#[derive(Debug)]
pub struct Column<'a, T>
where
    T: System,
{
    children: IntoIter<Element<'a, T>>,
}

impl<'a, T> Column<'a, T>
where
    T: System,
{
    pub fn new(children: Vec<Element<'a, T>>) -> Self {
        Self {
            children: children.into_iter(),
        }
    }
}

impl<'a, T> Iterator for Column<'a, T>
where
    T: System,
{
    type Item = Element<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.children.next()
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
    /// ```text
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
    /// ```text
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
pub struct RenderState<'a, T>
where
    T: System,
{
    pub(crate) root: Element<'a, T>,
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
