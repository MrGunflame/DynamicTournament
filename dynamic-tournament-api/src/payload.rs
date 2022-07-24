use serde::{Deserialize, Serialize};

/// A payload of either `T` or `Vec<T>`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Payload<T> {
    Single(T),
    Multiple(Vec<T>),
}

impl<T> Payload<T> {
    /// Returns the number of elements in the `Payload`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let payload = Payload::Single(1);
    ///
    /// assert_eq!(payload.len(), 1);
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let payload = Payload::Multiple(vec![1, 2, 3]);
    ///
    /// assert_eq!(payload.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multiple(v) => v.len(),
        }
    }

    /// Returns `true` if the `Payload` contains no elements.
    ///
    /// `is_empty` returning `true` implies that `self` is `Payload::Multiple` with no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let payload = Payload::Single(1);
    ///
    /// assert!(!payload.is_empty());
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let payload = Payload::Multiple(Vec::<i32>::new());
    ///
    /// assert!(payload.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the elements in the payload.
    ///
    /// # Example
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let payload = Payload::Single(1);
    /// let mut iter = payload.iter();
    ///
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), None);
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let payload = Payload::Multiple(vec![1, 2, 3]);
    /// let mut iter = payload.iter();
    ///
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&3));
    /// assert_eq!(iter.next(), None);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            payload: self,
            pos: 0,
        }
    }

    /// Returns a mutable iterator over the elements in the payload.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let mut payload = Payload::Single(1);
    ///
    /// for elem in payload.iter_mut() {
    ///     *elem *= 2;
    /// }
    ///
    /// assert_eq!(payload, Payload::Single(2));
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_api::Payload;
    /// let mut payload = Payload::Multiple(vec![1, 2, 3]);
    ///
    /// for elem in payload.iter_mut() {
    ///     *elem *= 2;
    /// }
    ///
    /// assert_eq!(payload, Payload::Multiple(vec![2, 4, 6]));
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            payload: self,
            pos: 0,
        }
    }
}

impl<T> From<T> for Payload<T> {
    #[inline]
    fn from(v: T) -> Self {
        Self::Single(v)
    }
}

impl<T> From<Vec<T>> for Payload<T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Self::Multiple(v)
    }
}

impl<'a, T> IntoIterator for &'a Payload<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Payload<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over the elements of a [`Payload`].
///
/// `Iter` is created by [`iter`].
///
/// [`iter`]: Payload::iter
#[derive(Debug)]
pub struct Iter<'a, T> {
    payload: &'a Payload<T>,
    pos: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &self.payload {
            Payload::Single(v) => match self.pos {
                0 => {
                    self.pos += 1;
                    Some(v)
                }
                _ => None,
            },
            Payload::Multiple(vec) => match vec.get(self.pos) {
                Some(v) => {
                    self.pos += 1;
                    Some(v)
                }
                None => None,
            },
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.payload.len() - self.pos;
        (n, Some(n))
    }
}

/// A mutable iterator over the elements of a [`Payload`].
///
/// `IterMut` is created by [`iter_mut`].
///
/// [`iter_mut`]: Payload::iter_mut
#[derive(Debug)]
pub struct IterMut<'a, T> {
    payload: &'a mut Payload<T>,
    pos: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let item = match &mut self.payload {
            Payload::Single(v) => match self.pos {
                0 => {
                    self.pos += 1;
                    Some(v)
                }
                _ => None,
            },
            Payload::Multiple(vec) => match vec.get_mut(self.pos) {
                Some(v) => {
                    self.pos += 1;

                    Some(v)
                }
                None => None,
            },
        }?;

        // Extend the lifetime.
        // SAFETY: IterMut has the same lifetime as the referenced payload.
        unsafe { Some(std::mem::transmute(item)) }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.payload {
            Payload::Single(_) => (1 - self.pos, Some(1 - self.pos)),
            Payload::Multiple(vec) => (vec.len() - self.pos, Some(vec.len() - self.pos)),
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    #[inline]
    fn len(&self) -> usize {
        self.payload.len() - self.pos
    }
}
