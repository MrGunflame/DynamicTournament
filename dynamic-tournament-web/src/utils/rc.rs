use std::ops::Deref;
use std::rc::Rc as RcInner;

/// A reference-counting pointer to `T` with an efficient [`PartialEq`] implementation.
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Rc<T>(RcInner<T>);

impl<T> Rc<T> {
    /// Creates a new `Rc`.
    #[inline]
    pub fn new(value: T) -> Self {
        Self(RcInner::new(value))
    }

    /// Returns `true` if both `Rc`s point to the same allocation.
    ///
    /// Note that the [`PartialEq`] implementation uses `ptr_eq`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use crate::utils::Rc;
    ///
    /// let ptr = Rc::new(1);
    /// let same_ptr = ptr.clone();
    /// let other_ptr = Rc::new(1);
    ///
    /// assert!(Rc::ptr_eq(ptr, same_ptr));
    /// assert!(!Rc::ptr_eq(ptr, other_ptr));
    /// ```
    #[inline]
    pub fn ptr_eq(this: &Rc<T>, other: &Rc<T>) -> bool {
        RcInner::ptr_eq(&this.0, &other.0)
    }
}

impl<T> Clone for Rc<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Deref for Rc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> PartialEq for Rc<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        Self::ptr_eq(self, other)
    }
}

impl<T> Eq for Rc<T> {}
