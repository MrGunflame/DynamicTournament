pub mod login;
pub mod logout;
pub mod not_found;
pub mod tournament;
pub mod tournamentlist;

use std::rc::Rc;

use crate::components::bracket::Bracket;
use crate::components::config_provider::ConfigProvider;
use crate::components::movable_boxed::MovableBoxed;
use crate::components::providers::{ClientProvider, Provider};
use crate::{render_data, Data, DataResult};

use yew::prelude::*;
use yew_router::prelude::*;
use yew_router::Routable;

use login::Login;
use logout::Logout;

use not_found::NotFound;

use dynamic_tournament_api::tournament::TournamentId;
use dynamic_tournament_api::Client;

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
                    <BrowserRouter>
                        <div class="main-wrapper">
                            <div>
                                <div class="navbar">
                                    <ul>
                                        <li><Link<Route> to={Route::Index}>{ "Home" }</Link<Route>></li>
                                        <li><Link<Route> to={Route::TournamentList}>{ "Tournaments" }</Link<Route>></li>
                                        <li><Link<Route> to={Route::Login}>{ "Login" }</Link<Route>></li>
                                        <li><Link<Route> to={Route::Logout}>{ "Logout" }</Link<Route>></li>
                                    </ul>
                                </div>
                                <div class="main">
                                    <Switch<Route> render={Switch::render(switch)} />
                                </div>
                                <div id="popup-host"></div>
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
            </ConfigProvider>
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
    #[at("/tournament")]
    TournamentList,
    #[at("/tournament/:id")]
    TournamentR { id: u64 },
    #[at("/tournament/:id/:s")]
    Tournament { id: u64 },
    #[at("/tournament/:id/teams/:s")]
    TournamentTeam { id: u64 },
    #[at("/embed/tournament/:id/bracket")]
    Embed { id: u64 },
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
            <tournament::Tournament id={TournamentId(*id)} />
        },
        Route::Tournament { id } => html! {
            <tournament::Tournament id={TournamentId(*id)} />
        },
        Route::TournamentTeam { id } => html! {
            <tournament::Tournament id={TournamentId(*id)} />
        },
        Route::Embed { id } => html! {
            <Embed id={TournamentId(*id)} />
        },
    }
}

pub struct Embed {
    data: Data<(
        Rc<dynamic_tournament_api::tournament::Tournament>,
        Option<Rc<dynamic_tournament_api::tournament::Bracket>>,
    )>,
}

impl Component for Embed {
    type Message = Msg;
    type Properties = EmbedProps;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::take(ctx);
        let id = ctx.props().id;

        ctx.link().send_future(async move {
            async fn fetch_data(
                client: Client,
                id: TournamentId,
            ) -> DataResult<(
                Rc<dynamic_tournament_api::tournament::Tournament>,
                Option<Rc<dynamic_tournament_api::tournament::Bracket>>,
            )> {
                let client = client.tournaments();

                let data = client.get(id).await?;

                let bracket = match client.bracket(id).get().await {
                    Ok(bracket) => Some(Rc::new(bracket)),
                    Err(_) => None,
                };

                Ok((Rc::new(data), bracket))
            }

            let data = Some(fetch_data(client, id).await);

            Msg::Update(data)
        });

        Self { data: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(data) => {
                self.data = data;

                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        render_data(&self.data, |(data, _)| {
            let tournament = data.clone();

            html! {
                <MovableBoxed classes="bracket-fullscreen">
                    <Bracket {tournament} />
                </MovableBoxed>
            }
        })
    }
}

#[derive(PartialEq, Properties)]
pub struct EmbedProps {
    id: TournamentId,
}

pub enum Msg {
    Update(
        Data<(
            Rc<dynamic_tournament_api::tournament::Tournament>,
            Option<Rc<dynamic_tournament_api::tournament::Bracket>>,
        )>,
    ),
}
