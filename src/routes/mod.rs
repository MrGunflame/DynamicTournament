pub mod login;
pub mod logout;
pub mod tournament;

use crate::components::config_provider::ConfigProvider;
use crate::components::providers::AuthProvider;

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::Routable;

use login::Login;
use logout::Logout;

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
                    <BrowserRouter>
                        <div class="navbar">
                            <ul>
                                <li><Link<Route> to={Route::Index}>{ "Home" }</Link<Route>></li>
                                <li><Link<tournament::Route> to={tournament::Route::Bracket}>{ "Bracket" }</Link<tournament::Route>></li>
                                <li><Link<tournament::Route> to={tournament::Route::Teams}>{ "Teams" }</Link<tournament::Route>></li>
                                <li><Link<Route> to={Route::Login}>{ "Login" }</Link<Route>></li>
                                <li><Link<Route> to={Route::Logout}>{ "Logout" }</Link<Route>></li>
                            </ul>
                        </div>
                        <Switch<Route> render={Switch::render(switch)} />
                    </BrowserRouter>
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
    #[at("/tournament/:s")]
    Tournament,
    #[at("/tournament/teams/:s")]
    TournamentTeam,
}

pub fn switch(route: &Route) -> Html {
    match route {
        Route::Index => html! { "this is index" },
        Route::Login => html! { <Login /> },
        Route::Logout => html! { <Logout /> },
        Route::NotFound => html! { "404" },
        Route::Tournament => html! {
            <tournament::Tournament />
        },
        Route::TournamentTeam => html! {
            <tournament::Tournament />
        },
    }
}
