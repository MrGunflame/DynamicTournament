use yew::prelude::*;
use yew_router::components::Redirect;

use super::Route;
use crate::components::providers::auth::Auth;

use gloo_storage::Storage;

pub struct Logout;

impl Component for Logout {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (auth, _) = ctx
            .link()
            .context::<Auth>(Callback::noop())
            .expect("No AuthContext provided");

        gloo_storage::LocalStorage::delete("http_auth_data");

        let mut inner = auth.inner.lock().unwrap();
        *inner = None;

        html! {
            <Redirect<Route> to={Route::Index} />
        }
    }
}
