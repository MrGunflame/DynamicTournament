mod api;
mod components;
mod consts;
mod logger;
mod routes;
mod services;
mod statics;
mod utils;

use yew::start_app_in_element;

use routes::App;

use consts::{MOUNTPOINT, TITLE_BASE};
use statics::Config;

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    // SAFETY: Called from a single threaded context. No race conditions can occur.
    unsafe {
        logger::init();
    }

    // SAFETY: There are no references to the config.
    unsafe {
        statics::set_config(Config {
            api_base: "http://localhost:3030".into(),
            root: "".into(),
        });
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
