mod fetch_data;

pub use fetch_data::FetchData;
use web_sys::Document;

pub fn document() -> Document {
    web_sys::window().unwrap().document().unwrap()
}
