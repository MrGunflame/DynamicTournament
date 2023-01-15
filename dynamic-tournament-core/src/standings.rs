use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::iter::FusedIterator;

#[derive(Clone, Debug)]
pub struct Standings {
    entries: Vec<Entry>,
    keys: Vec<Cow<'static, str>>,
}

impl Standings {
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            inner: self,
            next: 0,
        }
    }

    #[inline]
    pub fn keys(&self) -> Keys<'_> {
        Keys {
            inner: self,
            next: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Builder {
    keys: Vec<Cow<'static, str>>,
    entries: Vec<Entry>,
}

impl Builder {
    #[inline]
    pub const fn new() -> Self {
        Self {
            keys: Vec::new(),
            entries: Vec::new(),
        }
    }

    #[inline]
    pub fn key<K>(&mut self, key: K) -> &mut Self
    where
        K: Into<Cow<'static, str>>,
    {
        self.keys.push(key.into());
        self
    }

    pub fn entry<F>(&mut self, index: usize, f: F) -> &mut Self
    where
        F: FnOnce(&mut EntryBuilder),
    {
        let mut builder = EntryBuilder::new(index);
        f(&mut builder);
        self.entries.push(builder.build());
        self
    }

    #[inline]
    pub fn build(self) -> Standings {
        Standings {
            entries: self.entries,
            keys: self.keys,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntryBuilder {
    index: usize,
    values: Vec<EntryValue>,
}

impl EntryBuilder {
    #[inline]
    const fn new(index: usize) -> Self {
        Self {
            index,
            values: Vec::new(),
        }
    }

    #[inline]
    pub fn value<V>(&mut self, value: V) -> &mut Self
    where
        V: Into<EntryValue>,
    {
        self.values.push(value.into());
        self
    }

    #[inline]
    fn build(self) -> Entry {
        Entry {
            index: self.index,
            values: self.values,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Iter<'a> {
    inner: &'a Standings,
    next: usize,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Entry;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.inner.entries.get(self.next)?;
        self.next += 1;
        Some(entry)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.entries.len() - self.next
    }
}

impl<'a> FusedIterator for Iter<'a> {}

#[derive(Clone, Debug)]
pub struct Keys<'a> {
    inner: &'a Standings,
    next: usize,
}

impl<'a> Iterator for Keys<'a> {
    type Item = &'a str;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let key = self.inner.keys.get(self.next)?;
        self.next += 1;
        Some(key)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'a> ExactSizeIterator for Keys<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.keys.len() - self.next
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub index: usize,
    pub values: Vec<EntryValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntryValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    F64(f64),
    Str(Cow<'static, str>),
}

impl Display for EntryValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool(val) => Display::fmt(val, f),
            Self::I64(val) => Display::fmt(val, f),
            Self::U64(val) => Display::fmt(val, f),
            Self::F64(val) => Display::fmt(val, f),
            Self::Str(val) => Display::fmt(val, f),
        }
    }
}

impl From<bool> for EntryValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for EntryValue {
    #[inline]
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u64> for EntryValue {
    #[inline]
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl From<f64> for EntryValue {
    #[inline]
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}

impl From<&'static str> for EntryValue {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self::Str(Cow::Borrowed(value))
    }
}

impl From<String> for EntryValue {
    #[inline]
    fn from(value: String) -> Self {
        Self::Str(value.into())
    }
}

impl EntryValue {
    #[inline]
    pub const fn kind(&self) -> EntryKind {
        match self {
            Self::Bool(_) => EntryKind::Bool,
            Self::I64(_) => EntryKind::I64,
            Self::U64(_) => EntryKind::U64,
            Self::F64(_) => EntryKind::F64,
            Self::Str(_) => EntryKind::Str,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EntryKind {
    Bool,
    I64,
    U64,
    F64,
    Str,
}
