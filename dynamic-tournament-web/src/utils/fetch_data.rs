use crate::components::error::Error;
use crate::components::loader::Loader;

use yew::{html, Html};

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A wrapper around an `Option<Result<T>>`.
#[derive(Debug)]
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

impl<T> From<Result<T, BoxError>> for FetchData<T> {
    fn from(res: Result<T, BoxError>) -> Self {
        Self { inner: Some(res) }
    }
}

impl<T> From<T> for FetchData<T> {
    fn from(value: T) -> Self {
        Self {
            inner: Some(Ok(value)),
        }
    }
}
