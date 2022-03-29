mod components;
mod routes;

use yew::prelude::*;

extern crate wee_alloc;

use yew::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn main() {
    yew::start_app::<crate::routes::tournament::Tournament>();
}

pub struct App {}

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <crate::routes::tournament::Tournament />
        }
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct Config {
    api_url: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct MatchmakerInput {
    teams: Vec<Team>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct Team {
    pub name: String,
    pub memberCount: u64,
    pub players: Vec<Player>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Player {
    #[serde(rename = "accountName")]
    account_name: String,
    role: i64,
    discord: String,
}

pub enum Msg {
    AddOne,
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
