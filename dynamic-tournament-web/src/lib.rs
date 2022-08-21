mod api;
mod components;
mod consts;
mod logger;
mod routes;
mod services;
mod statics;
mod utils;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use yew::start_app_in_element;

use routes::App;

use consts::{MOUNTPOINT, TITLE_BASE};

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// The global config.
///
/// This instance always lives for the lifetime of the program.
///
/// # Safety
///
/// An `Config` instance is always expected to have a `'static` lifetime. Some methods make use of
/// this assumption to provide safe methods.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    // We never need to resize this, so `Box<str>` saves us 1 * usize of space.
    api_base: Box<str>,
}

impl Config {
    /// Returns the defined api_base.
    #[inline]
    pub fn api_base(&self) -> &'static str {
        &self.static_ref().api_base
    }

    /// Converts a `&Config` reference into a `&'static Config` reference.
    #[inline]
    fn static_ref(&self) -> &'static Self {
        // SAFETY: The caller must guarantee that `self` has a `'static` lifetime.
        unsafe { std::mem::transmute(self) }
    }
}

#[wasm_bindgen]
pub fn main(config: &JsValue) {
    // SAFETY: Called from a single threaded context. No race conditions can occur.
    unsafe {
        logger::init();
    }

    let config: Config = config.into_serde().unwrap();
    // SAFETY: There are no references to the config.
    unsafe {
        statics::set_config(config);
    }

    let document = web_sys::window()
        .expect("No window")
        .document()
        .expect("No Document");

    let element = match MOUNTPOINT {
        Mountpoint::Body => document.body().expect("No document body found").into(),
        Mountpoint::Element(id) => document
            .get_element_by_id(id)
            .expect("No element with the given id found"),
    };

    start_app_in_element::<App>(element);
}

#[derive(Copy, Clone, Debug)]
pub enum Mountpoint {
    Body,
    Element(&'static str),
}

pub struct Title;

impl Title {
    pub fn set(title: &str) {
        let document = web_sys::window().unwrap().document().unwrap();

        document.set_title(&format!("{} - {}", title, TITLE_BASE))
    }

    pub fn clear() {
        let document = web_sys::window().unwrap().document().unwrap();
        document.set_title(TITLE_BASE);
    }
}
