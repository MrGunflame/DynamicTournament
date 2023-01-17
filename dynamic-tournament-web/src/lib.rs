#![deny(unused_crate_dependencies)]
// Fix for html! macro doing unit value assignments. (yew 0.19.3)
#![allow(clippy::let_unit_value)]

mod api;
mod components;
mod consts;
mod logger;
mod routes;
mod services;
mod statics;
mod utils;

use wasm_bindgen::prelude::*;
use yew::start_app_in_element;

pub use statics::Config;

use routes::App;

use consts::TITLE_BASE;

#[wasm_bindgen]
pub fn run(config: JsValue) {
    let config = serde_wasm_bindgen::from_value(config).expect("Failed to parse config");
    run_with_config(config);
}

pub fn run_with_config(config: Config) {
    // SAFETY: Called from a single threaded context. No race conditions can occur.
    unsafe {
        logger::init();
    }

    let document = web_sys::window()
        .expect("No window")
        .document()
        .expect("No Document");

    let element = match document.get_element_by_id(config.mountpoint()) {
        Some(element) => element,
        None => {
            log::error!("Cannot find element with id {}", config.mountpoint());
            log::error!("Fatal error: Failed to mount app");

            return;
        }
    };

    // SAFETY: There are no references to the config.
    unsafe {
        statics::set_config(config);
    }

    start_app_in_element::<App>(element);
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
