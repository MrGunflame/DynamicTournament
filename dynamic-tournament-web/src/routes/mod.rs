pub mod login;
pub mod logout;
pub mod not_found;
pub mod systems;
pub mod tournamentlist;
pub mod tournaments;

use crate::components::errorlog::ErrorLog;
use crate::components::providers::ClientProvider;
use crate::components::Navbar;
use crate::utils::router::{self, PathBuf, Routable, Switch};

use dynamic_tournament_api::v3::id::TournamentId;
use yew::prelude::*;

use login::Login;
use logout::Logout;

use self::tournaments::Tournament;
use not_found::NotFound;

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        // Initialize the router.
        // SAFETY: Called from a single-threaded context. Since App is only
        // created once during the lifetime of the program, the value is never
        // overwritten without being dropped.
        unsafe {
            router::init();
        }

        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <ClientProvider>
                <div class="main-wrapper">
                    <div>
                        <Navbar />
                        <div class="dt-main">
                            <Switch<Route> render={Switch::render(switch)} />
                        </div>
                        <div id="dt-popup-host"></div>
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
            </ClientProvider>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Index,
    Login,
    Logout,
    Tournament { id: TournamentId },
    Systems,
    NotFound,
}

impl Routable for Route {
    fn from_path(path: &mut PathBuf) -> Option<Self> {
        match path.take().as_deref() {
            None => Some(Self::Index),
            Some("login") => Some(Self::Login),
            Some("logout") => Some(Self::Logout),
            Some("systems") => Some(Self::Systems),
            Some(s) => {
                let id = s.parse().ok()?;
                Some(Self::Tournament { id })
            }
        }
    }

    fn to_path(&self) -> String {
        match self {
            Self::Index => String::from("/"),
            Self::Login => String::from("/login"),
            Self::Logout => String::from("/logout"),
            Self::Systems => String::from("/systems"),
            Self::NotFound => String::from("/404"),
            Self::Tournament { id } => format!("/{}", id),
        }
    }

    fn not_found() -> Option<Self> {
        Some(Self::NotFound)
    }
}

pub fn switch(route: &Route) -> Html {
    match route {
        Route::Index => html! {
            <tournamentlist::TournamentList />
        },
        Route::Login => html! { <Login /> },
        Route::Logout => html! { <Logout /> },
        Route::Tournament { id } => html! {
            <Tournament id={*id} />
        },
        Route::Systems => html! {
            <systems::Systems />
        },
        Route::NotFound => html! {
            <NotFound />
        },
    }
}
