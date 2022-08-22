pub mod login;
pub mod logout;
pub mod not_found;
pub mod systems;
pub mod tournamentlist;
pub mod tournaments;

use crate::components::errorlog::ErrorLog;
use crate::components::providers::ClientProvider;
use crate::components::Navbar;
use crate::utils::router::{Path, Routable, Router, Switch};

use yew::prelude::*;

use login::Login;
use logout::Logout;

use not_found::NotFound;

use self::tournaments::Tournaments;

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
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
    fn from_path(path: &mut Path) -> Option<Self> {
        match path.take() {
            None => Some(Self::Index),
            Some("login") => Some(Self::Login),
            Some("logout") => Some(Self::Logout),
            Some("tournaments") => Some(Self::Tournaments),
            Some("systems") => Some(Self::Systems),
            Some(_) => None,
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
            <Tournaments />
        },
        Route::Systems => html! {
            <systems::Systems />
        },
        Route::NotFound => html! {
            <NotFound />
        },
    }
}
