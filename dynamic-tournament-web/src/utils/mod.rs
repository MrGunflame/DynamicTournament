mod fetch_data;
mod rc;
pub mod router;

pub use fetch_data::FetchData;
pub use rc::Rc;
use web_sys::{Document, History, Window};

#[inline]
pub fn window() -> Window {
    web_sys::window().expect("no window found")
}

/// Returns the root [`Document`].
///
/// # Panics
///
/// Panics if there is no [`Document`] in root window or no root window is present. This should
/// never be the case in a web environment.
pub fn document() -> Document {
    window().document().expect("no document present")
}

pub fn history() -> History {
    window().history().expect("no history")
}
