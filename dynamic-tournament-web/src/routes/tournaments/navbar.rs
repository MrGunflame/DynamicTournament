use dynamic_tournament_api::v3::id::TournamentId;
use yew::{html, Component, Context, Html, Properties};

use super::tournament::Route;
use crate::api::{Action, State};
use crate::components::providers::{ClientProvider, Provider};
use crate::utils::router::{Link, Routable};

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament_id: TournamentId,
    pub route: Route,
}

#[derive(Debug)]
pub struct Navbar {
    state: State,
}

impl Component for Navbar {
    type Message = Action;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);
        let state = client.state();

        let link = ctx.link().clone();
        ctx.link().send_future_batch(async move {
            loop {
                let action = client.changed().await;
                link.send_message(action);
            }
        });

        Self { state }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.state = match msg {
            Action::Login => State::LoggedIn,
            Action::Logout => State::LoggedOut,
        };

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut links = vec![
            (Route::Index, "Overview"),
            (Route::Brackets, "Brackets"),
            (Route::Entrants, "Entrants"),
        ];

        if self.state == State::LoggedIn {
            links.push((Route::Admin, "Admin"));
        }

        let links: Html = links
            .into_iter()
            .map(|(route, name)| {
                let classes = if route == ctx.props().route {
                    "active"
                } else {
                    ""
                };

                let to = format!("/{}{}", ctx.props().tournament_id, route.to_path());

                html! {
                    <li><Link {classes} {to}>{ name }</Link></li>
                }
            })
            .collect();

        html! {
            <>
                <div class="navbar">
                    <ul>
                        { links }
                    </ul>
                </div>
            </>
        }
    }
}
