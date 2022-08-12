pub mod login;
pub mod logout;
pub mod not_found;
pub mod systems;
pub mod tournamentlist;
pub mod tournaments;

use crate::components::config_provider::ConfigProvider;
use crate::components::errorlog::ErrorLog;
use crate::components::providers::ClientProvider;
use crate::components::Navbar;
use crate::utils::router::{Routable, Router, Switch};

use yew::prelude::*;

use login::Login;
use logout::Logout;

use not_found::NotFound;

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ConfigProvider>
                <ClientProvider>
                    <Router>
                        <div class="main-wrapper">
                            <div>
                                <Navbar />
                                <div class="main">
                                    <Switch<Route> render={Switch::render(switch)} />
                                </div>
                                <div id="popup-host"></div>
                                <ErrorLog />
                            </div>
                            <div class="footer">
                                <p>
                                    { "This viewer is still in an early stage, please report issues on " }
                                    <a href="https://github.com/MrGunflame/DynamicTournament/issues">{ "Github" }</a>
                                    { " or to MagiiTech#0534 on Discord." }
                                </p>
                                <a href="/privacy.html">{ "Privacy Policy" }</a>
                            </div>
                        </div>
                    </Router>
                </ClientProvider>
            </ConfigProvider>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Index,
    Login,
    Logout,
    Tournaments,
    Systems,
    NotFound,
}

impl Routable for Route {
    fn from_path(path: &str) -> Option<Self> {
        match path {
            "/" => Some(Self::Index),
            "/login" => Some(Self::Login),
            "/logout" => Some(Self::Logout),
            "/tournaments" => Some(Self::Tournaments),
            "/systems" => Some(Self::Systems),
            _ => None,
        }
    }

    fn to_path(&self) -> String {
        match self {
            Self::Index => String::from("/"),
            Self::Login => String::from("/login"),
            Self::Logout => String::from("/logout"),
            Self::Tournaments => String::from("/tournaments"),
            Self::Systems => String::from("/systems"),
            Self::NotFound => String::from("/404"),
        }
    }

    fn not_found() -> Option<Self> {
        Some(Self::NotFound)
    }
}

pub fn switch(route: &Route) -> Html {
    match route {
        Route::Index => html! { "this is index" },
        Route::Login => html! { <Login /> },
        Route::Logout => html! { <Logout /> },
        Route::Tournaments => html! {
            <tournamentlist::TournamentList />
        },
        Route::Systems => html! {
            <systems::Systems />
        },
        Route::NotFound => html! {
            <NotFound />
        },
    }
}
