mod components;
mod consts;
mod logger;
mod routes;
mod services;
mod utils;

use yew::prelude::*;
use yew::start_app_in_element;

use routes::App;

use consts::{MOUNTPOINT, TITLE_BASE};

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    // SAFETY: Called from a single threaded context. No race conditions can occur.
    unsafe {
        logger::init();
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

pub fn render_data<T, F>(
    data: &Option<Result<T, Box<dyn std::error::Error + Send + Sync>>>,
    f: F,
) -> Html
where
    F: FnOnce(&T) -> Html,
{
    match data {
        Some(data) => match data {
            Ok(data) => f(data),
            Err(err) => html! {
                <crate::components::error::Error error={err.to_string()} />
            },
        },
        None => html! {
            <crate::components::loader::Loader />
        },
    }
}

pub type Data<T> = Option<Result<T, Box<dyn std::error::Error + 'static + Send + Sync>>>;
pub type DataResult<T> = Result<T, Box<dyn std::error::Error + 'static + Send + Sync>>;

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
