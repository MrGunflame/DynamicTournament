mod api;
mod components;
mod routes;

use yew::prelude::*;

use crate::components::config_provider::ConfigProvider;

extern crate wee_alloc;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    yew::start_app::<App>();
}

pub struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ConfigProvider>
                <crate::routes::tournament::Tournament />
            </ConfigProvider>
        }
    }
}

pub fn render_data<T, F>(data: &Option<Result<T, Box<dyn std::error::Error>>>, f: F) -> Html
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

pub type Data<T> = Option<Result<T, Box<dyn std::error::Error + 'static>>>;
pub type DataResult<T> = Result<T, Box<dyn std::error::Error + 'static>>;
