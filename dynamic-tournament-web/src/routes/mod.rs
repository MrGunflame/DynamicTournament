pub mod login;
pub mod logout;
pub mod tournament;
pub mod tournamentlist;

use crate::components::config_provider::ConfigProvider;
use crate::components::providers::{AuthProvider, ClientProvider};

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::Routable;

use login::Login;
use logout::Logout;

use dynamic_tournament_api::tournament::TournamentId;

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
                <AuthProvider>
                    <ClientProvider>
                        <BrowserRouter>
                            <div class="navbar">
                                <ul>
                                    <li><Link<Route> to={Route::Index}>{ "Home" }</Link<Route>></li>
                                    <li><Link<Route> to={Route::TournamentList}>{ "Tournaments" }</Link<Route>></li>
                                    <li><Link<Route> to={Route::Login}>{ "Login" }</Link<Route>></li>
                                    <li><Link<Route> to={Route::Logout}>{ "Logout" }</Link<Route>></li>
                                </ul>
                            </div>
                            <Switch<Route> render={Switch::render(switch)} />
                        </BrowserRouter>
                    </ClientProvider>
                </AuthProvider>
            </ConfigProvider>
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Routable)]
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
    #[at("/tournament")]
    TournamentList,
    #[at("/tournament/:id")]
    TournamentR { id: u64 },
    #[at("/tournament/:id/:s")]
    Tournament { id: u64 },
    #[at("/tournament/:id/teams/:s")]
    TournamentTeam { id: u64 },
}

pub fn switch(route: &Route) -> Html {
    match route {
        Route::Index => html! { "this is index" },
        Route::Login => html! { <Login /> },
        Route::Logout => html! { <Logout /> },
        Route::NotFound => html! { "404" },
        Route::TournamentList => html! {
            <tournamentlist::TournamentList />
        },
        Route::TournamentR { id } => html! {
            <tournament::Tournament id={TournamentId(*id)} />
        },
        Route::Tournament { id } => html! {
            <tournament::Tournament id={TournamentId(*id)} />
        },
        Route::TournamentTeam { id } => html! {
            <tournament::Tournament id={TournamentId(*id)} />
        },
    }
}
