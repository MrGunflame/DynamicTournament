mod fetch_data;

pub use fetch_data::FetchData;
use web_sys::Document;

/// Returns the root [`Document`].
///
/// # Panics
///
/// Panics if there is no [`Document`] in root window or no root window is present. This should
/// never be the case in a web environment.
pub fn document() -> Document {
    web_sys::window()
        .expect("no window present")
        .document()
        .expect("no document present")
}
