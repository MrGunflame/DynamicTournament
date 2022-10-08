mod fetch_data;
mod rc;
pub mod router;

use std::process::abort;

pub use fetch_data::FetchData;
pub use rc::Rc;
use web_sys::{Document, History, Window};

/// Returns the root [`Window`]. This function aborts when no window is present.
#[inline]
pub fn window() -> Window {
    match web_sys::window() {
        Some(window) => window,
        None => abort(),
    }
}

/// Returns the root [`Document`]. This function aborts when no document is present.
#[inline]
pub fn document() -> Document {
    match window().document() {
        Some(document) => document,
        None => abort(),
    }
}

/// Returns the window [`History`]. This function aborts when no history is present.
#[inline]
pub fn history() -> History {
    match window().history() {
        Ok(history) => history,
        Err(_) => abort(),
    }
}
