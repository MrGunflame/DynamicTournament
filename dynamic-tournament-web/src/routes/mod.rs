pub mod login;
pub mod logout;
pub mod not_found;
pub mod systems;
pub mod tournamentlist;
pub mod tournaments;

use crate::components::errorlog::ErrorLog;
use crate::components::providers::ClientProvider;
use crate::components::Navbar;

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::Routable;

use login::Login;
use logout::Logout;

use not_found::NotFound;

use dynamic_tournament_api::v3::id::TournamentId;

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
                <BrowserRouter>
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
                </BrowserRouter>
            </ClientProvider>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Routable)]
pub enum Route {
    #[at("/")]
    Index,
    #[at("/login")]
    Login,
    #[at("/logout")]
    Logout,
    #[not_found]
    #[at("/404")]
    NotFound,
    #[at("/tournaments")]
    TournamentList,
    #[at("/tournaments/:id")]
    TournamentR { id: u64 },
    #[at("/tournaments/:id/:s")]
    Tournament { id: u64 },
    #[at("/tournaments/:id/:name/brackets")]
    TournamentBrackets { id: u64 },
    #[at("/tournaments/:id/:name/brackets/:s/:s")]
    TournamentBracket { id: u64 },
    #[at("/tournaments/:id/:name/entrants")]
    TournamentTeams { id: u64 },
    #[at("/tournaments/:id/:name/entrants/:s")]
    TournamentTeam { id: u64 },
    #[at("/tournaments/:id/:name/admin")]
    Admin { id: u64 },
    #[at("/systems")]
    Systems,
}

pub fn switch(route: &Route) -> Html {
    match route {
        Route::Index => html! { "this is index" },
        Route::Login => html! { <Login /> },
        Route::Logout => html! { <Logout /> },
        Route::NotFound => html! { <NotFound /> },
        Route::TournamentList => html! {
            <tournamentlist::TournamentList />
        },
        Route::TournamentR { id } => html! {
            <tournaments::Tournament id={TournamentId(*id)} />
        },
        Route::Tournament { id } => html! {
            <tournaments::Tournament id={TournamentId(*id)} />
        },
        Route::TournamentTeam { id } => html! {
            <tournaments::Tournament id={TournamentId(*id)} />
        },
        Route::TournamentBracket { id } => html! {
            <tournaments::Tournament id={TournamentId(*id)} />
        },
        Route::TournamentBrackets { id } => html! {
            <tournaments::Tournament id={TournamentId(*id)} />
        },
        Route::TournamentTeams { id } => html! {
            <tournaments::Tournament id={TournamentId(*id)} />
        },
        Route::Admin { id } => html! {
            <tournaments::Tournament id={TournamentId(*id)} />
        },
        Route::Systems => html! {
            <systems::Systems />
        },
    }
}
