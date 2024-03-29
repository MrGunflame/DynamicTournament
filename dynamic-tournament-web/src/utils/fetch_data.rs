use crate::components::error::Error;
use crate::components::loader::Loader;

use yew::{html, Html};

use std::rc::Rc;

// We use a `Rc` instead of a `Box` here so we can avoid cloning errors when going from
// `FetchData<T>` to `FetchData<&T>` or `FetchData<&mut T>`.
pub type BoxError = Rc<dyn std::error::Error + Send + Sync + 'static>;

/// A wrapper around an `Option<Result<T>>`.
#[derive(Clone, Debug)]
pub struct FetchData<T> {
    inner: Option<Result<T, BoxError>>,
}

impl<T> FetchData<T> {
    /// Creates a new `FetchData` with an uninitialized state.
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new `FetchData` with an initialized `Ok` state.
    pub fn new_with_value(value: T) -> Self {
        Self {
            inner: Some(Ok(value)),
        }
    }

    pub fn from_err<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self {
            inner: Some(Err(Rc::new(err))),
        }
    }

    fn from_raw_err(err: Rc<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self {
            inner: Some(Err(err)),
        }
    }

    #[allow(unused)]
    pub fn as_ref(&self) -> FetchData<&T> {
        match &self.inner {
            Some(res) => match res {
                Ok(ref value) => FetchData::from(value),
                Err(err) => FetchData::from(Err(err.clone())),
            },
            None => FetchData::new(),
        }
    }

    #[allow(unused)]
    pub fn as_mut(&mut self) -> FetchData<&mut T> {
        match self.inner {
            Some(ref mut res) => match res {
                Ok(ref mut value) => FetchData::from(value),
                Err(err) => FetchData::from(Err(err.clone())),
            },
            None => FetchData::new(),
        }
    }

    /// Returns `true` if the `FetchData` has an initialized value.
    #[allow(unused)]
    pub fn has_value(&self) -> bool {
        match self.inner {
            Some(ref res) => res.is_ok(),
            None => false,
        }
    }

    pub fn zip<'a, U>(&'a self, other: &'a FetchData<U>) -> FetchData<(&'a T, &'a U)> {
        let val = match &self.inner {
            Some(res) => match res {
                Ok(val) => val,
                Err(err) => return FetchData::from_raw_err(err.clone()),
            },
            None => return FetchData::new(),
        };

        match &other.inner {
            Some(res) => match res {
                Ok(other) => FetchData::from((val, other)),
                Err(err) => FetchData::from_raw_err(err.clone()),
            },
            None => FetchData::new(),
        }
    }

    /// Maps a `FetchData<T>` to an `FetchData<U>`.
    #[inline]
    #[allow(unused)]
    pub fn map<U, F>(self, f: F) -> FetchData<U>
    where
        F: FnOnce(T) -> U,
    {
        FetchData::from(self.inner.map(|res| res.map(f)))
    }

    /// Unwraps the value `T` from `FetchData<T>`, panicking when it contains no `T` value.
    ///
    /// # Panics
    ///
    /// Panics if `self` has no value `T`.
    #[allow(unused)]
    #[inline]
    pub fn unwrap(self) -> T {
        self.inner.unwrap().unwrap()
    }

    pub fn unwrap_or_default(self) -> T
    where
        T: Default,
    {
        match self.inner {
            Some(res) => match res {
                Ok(val) => val,
                Err(_) => T::default(),
            },
            None => T::default(),
        }
    }

    /// Unwraps the value `T` from `FetchData<T>` without checking if it contains `T`.
    ///
    /// # Safety
    ///
    /// This method causes undefined behavoir if called on a value that is not `T`.
    #[allow(unused)]
    #[inline]
    pub unsafe fn unwrap_unchecked(self) -> T {
        self.inner.unwrap_unchecked().unwrap_unchecked()
    }

    pub fn render<F>(&self, f: F) -> Html
    where
        F: FnOnce(&T) -> Html,
    {
        match &self.inner {
            Some(res) => match res {
                Ok(value) => {
                    log::debug!("FetchData is initialized to an `Ok` value, rendering using `F`");

                    f(value)
                }
                Err(err) => {
                    log::debug!(
                        "FetchData is initialized to an `Err` value, rendering error component"
                    );

                    html! {
                        <Error error={err.to_string()} />
                    }
                }
            },
            None => {
                log::debug!("FetchData is `None`, rendering loader component");

                html! {
                    <Loader />
                }
            }
        }
    }
}

impl<T> Default for FetchData<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> From<Option<Result<T, BoxError>>> for FetchData<T> {
    fn from(opt: Option<Result<T, BoxError>>) -> Self {
        Self { inner: opt }
    }
}

impl<T> From<Option<Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>>>
    for FetchData<T>
{
    fn from(opt: Option<Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>>) -> Self {
        match opt {
            Some(v) => Self::from(v),
            None => Self::new(),
        }
    }
}

impl<T> From<Result<T, BoxError>> for FetchData<T> {
    fn from(res: Result<T, BoxError>) -> Self {
        Self { inner: Some(res) }
    }
}

impl<T> From<Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>> for FetchData<T> {
    fn from(res: Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>) -> Self {
        match res {
            Ok(v) => Self::new_with_value(v),
            Err(err) => {
                let err: BoxError = Rc::from(err);

                Self {
                    inner: Some(Err(err)),
                }
            }
        }
    }
}

impl<T> From<T> for FetchData<T> {
    fn from(value: T) -> Self {
        Self {
            inner: Some(Ok(value)),
        }
    }
}
