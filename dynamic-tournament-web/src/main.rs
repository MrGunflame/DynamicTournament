mod components;
mod routes;

use yew::prelude::*;

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    yew::start_app::<routes::App>();
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
