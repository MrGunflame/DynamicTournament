use yew::{html, Children, Component, Context, Html, Properties};

use super::providers::{ClientProvider, Provider};
use crate::utils::router::Redirect;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

/// Protect a page requiring authentication.
///
/// This component will only render its children if the user is authenticated and redirect to the
/// login page if he is not.
#[derive(Debug)]
pub struct Protected {
    _priv: (),
}

impl Component for Protected {
    type Message = ();
    type Properties = Props;

    #[inline]
    fn create(_ctx: &Context<Self>) -> Self {
        Self { _priv: () }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_authenticated = ClientProvider::get(ctx).is_authenticated();

        if is_authenticated {
            html! {
                for ctx.props().children.iter()
            }
        } else {
            html! {
                <Redirect to="/login" />
            }
        }
    }
}
