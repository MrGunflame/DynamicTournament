mod danger_zone;
mod entrants;
mod settings;

use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::{html, Component, Context, Html, Properties};
use yew_router::components::Redirect;

use crate::components::providers::{ClientProvider, Provider};
use crate::routes::Route;
use crate::utils::Rc;

use self::danger_zone::DangerZone;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

/// The root of the admin section of a tournament.
#[derive(Debug)]
pub struct Admin;

impl Component for Admin {
    type Message = ();
    type Properties = Props;

    #[inline]
    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let client = ClientProvider::get(ctx);

        if client.is_authenticated() {
            html! {
                <div>
                    <settings::Settings tournament={ctx.props().tournament.clone()} />
                    <entrants::Entrants tournament={ctx.props().tournament.clone()} />

                    <DangerZone tournament={ctx.props().tournament.clone()} />
                </div>
            }
        } else {
            html! {
                <Redirect<Route> to={Route::Login} />
            }
        }
    }
}
