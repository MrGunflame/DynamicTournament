use dynamic_tournament_api::auth::Flags;
use dynamic_tournament_api::v3::id::TournamentId;
use yew::{html, Component, Context, Html, Properties};

use super::tournament::Route;
use crate::components::Protected;
use crate::utils::router::{Link, Routable};

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament_id: TournamentId,
    pub route: Route,
}

#[derive(Debug)]
pub struct Navbar {
    _priv: (),
}

impl Component for Navbar {
    type Message = ();
    type Properties = Props;

    #[inline]
    fn create(_ctx: &Context<Self>) -> Self {
        Self { _priv: () }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let links = vec![
            (Route::Index, "Overview", None),
            (Route::Brackets, "Brackets", None),
            (Route::Entrants, "Entrants", None),
            (Route::Admin, "Admin", Some(Flags::ADMIN)),
        ];

        let links: Html = links
            .into_iter()
            .map(|(route, name, flags)| {
                let classes = if route == ctx.props().route {
                    "dt-active"
                } else {
                    ""
                };

                let to = format!("/{}{}", ctx.props().tournament_id, route.to_path());

                match flags {
                    Some(flags) => {
                        html! {
                            <Protected {flags}>
                                <li><Link {classes} {to}>{ name }</Link></li>
                            </Protected>
                        }
                    }
                    None => {
                        html! {
                            <li><Link {classes} {to}>{ name }</Link></li>
                        }
                    }
                }
            })
            .collect();

        html! {
            <>
                <div class="dt-navbar">
                    <ul>
                        { links }
                    </ul>
                </div>
            </>
        }
    }
}
