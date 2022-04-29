use std::ops::Deref;

/// An [`Option`]-like type with the same size as `T`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SmallOption<T>
where
    T: SmallOptionValue + PartialEq,
{
    value: T,
}

impl<T> SmallOption<T>
where
    T: SmallOptionValue + PartialEq,
{
    /// Creates a new `SmallOption` with the given `value`.
    ///
    /// # Panics
    ///
    /// Panics when `value` collides with the value of `T::NONE`.
    #[allow(unused)]
    pub fn new(value: T) -> Self {
        if value == T::NONE {
            panic!("Tried to create a SmallOption with the `NONE` value")
        } else {
            Self::new_unchecked(value)
        }
    }

    /// Creates a new `SmallOption` with the given `value` without checking whether it collides
    /// with the value of `T::NONE`.
    pub fn new_unchecked(value: T) -> Self {
        Self { value }
    }

    pub fn is_none(&self) -> bool {
        self.value == T::NONE
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

impl<T> Default for SmallOption<T>
where
    T: SmallOptionValue + PartialEq + Default,
{
    fn default() -> Self {
        Self::new_unchecked(T::default())
    }
}

pub trait SmallOptionValue {
    const NONE: Self;
}

impl SmallOptionValue for usize {
    const NONE: Self = usize::MAX;
}

impl<T> Deref for SmallOption<T>
where
    T: SmallOptionValue + PartialEq,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
