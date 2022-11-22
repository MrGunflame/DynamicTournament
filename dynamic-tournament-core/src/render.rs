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

/// An textual label attached to an [`Element`].
#[derive(Clone, Debug)]
pub struct Label<'a>(Cow<'a, str>);

impl<'a> Label<'a> {
    pub fn as_str(&'a self) -> &'a str {
        &self.0
    }
}

#[derive(Debug)]
pub enum Element<'a, T>
where
    T: System,
{
    Row(Row<'a, T>),
    Column(Column<'a, T>),
    Match(Match<'a, T>),
}

impl<'a, T> Element<'a, T>
where
    T: System,
{
    pub(crate) fn new<E>(inner: E) -> Self
    where
        E: Into<Element<'a, T>>,
    {
        inner.into()
    }

    pub fn kind(&self) -> ElementKind {
        match self {
            Self::Row(_) => ElementKind::Row,
            Self::Column(_) => ElementKind::Column,
            Self::Match(_) => ElementKind::Match,
        }
    }

    pub fn unwrap_row(self) -> Row<'a, T> {
        match self {
            Self::Row(val) => val,
            _ => panic!("called `unwrap_row` on an invalid ElementInner value"),
        }
    }

    pub fn unwrap_column(self) -> Column<'a, T> {
        match self {
            Self::Column(val) => val,
            _ => panic!("called `unwrap_column` on an invalid ElementInner value"),
        }
    }

    pub fn unwrap_match(self) -> Match<'a, T> {
        match self {
            Self::Match(val) => val,
            _ => panic!("called `unwrap_match`on an invalid ElementInner value"),
        }
    }
}

impl<'a, T> From<Row<'a, T>> for Element<'a, T>
where
    T: System,
{
    fn from(row: Row<'a, T>) -> Self {
        Self::Row(row)
    }
}

impl<'a, T> From<Column<'a, T>> for Element<'a, T>
where
    T: System,
{
    fn from(col: Column<'a, T>) -> Self {
        Self::Column(col)
    }
}

impl<'a, T> From<Match<'a, T>> for Element<'a, T>
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
    pub label: Option<Label<'a>>,
    pub position: Option<Position>,
    pub(crate) children: IntoIter<Element<'a, T>>,
}

impl<'a, T> Row<'a, T>
where
    T: System,
{
    pub(crate) fn new(children: Vec<Element<'a, T>>) -> Self {
        Self {
            label: None,
            position: None,
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
    pub label: Option<Label<'a>>,
    pub position: Option<Position>,
    pub(crate) children: IntoIter<Element<'a, T>>,
}

impl<'a, T> Column<'a, T>
where
    T: System,
{
    pub fn new(children: Vec<Element<'a, T>>) -> Self {
        Self {
            label: None,
            position: None,
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
    pub label: Option<Label<'a>>,
    pub position: Option<Position>,
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
