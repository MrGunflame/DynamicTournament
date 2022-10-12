use dynamic_tournament_api::auth::Flags;
use yew::{html, Children, Component, Context, Html, Properties};

use super::providers::{ClientProvider, Provider};
use crate::api::Action;
use crate::routes::not_found::NotFound;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    /// A list of [`Flags`] that need to be active. If `None` the page only protects
    /// if there is no token set.
    #[prop_or_default]
    pub flags: Option<Flags>,
    /// An action that is executed when a client is not allowed to view the protected content.
    #[prop_or_default]
    pub action: ForbiddenAction,
    pub children: Children,
}

/// An action that is executed when the client is not allowed to see the contents of a `Protected`
/// component. The default action of `None` displays nothing.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ForbiddenAction {
    /// Display nothing if the client is forbidden.
    #[default]
    None,
    /// Displays a `404 Not Found` instead of the children if the client is forbidden.
    NotFound,
}

/// Protects all children requiring authentication.
///
/// Using `Protected` is the preferred way to hide certain components for unauthenticated users.
/// The children of `Protected` if the user is logged and has all flags specified in [`Props`].
///
/// `Protected` automatically rerenders when the authorization state of the API client changes.
#[derive(Debug)]
pub struct Protected {
    _priv: (),
}

impl Component for Protected {
    type Message = Action;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let link = ctx.link().clone();
        ctx.link().send_future_batch(async move {
            loop {
                let action = client.changed().await;
                link.send_message(action);
            }
        });

        Self { _priv: () }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let is_allowed = match ClientProvider::get(ctx).authorization().refresh_token() {
            Some(token) => match ctx.props().flags {
                Some(other) => token.claims().flags.intersects(other),
                None => true,
            },
            None => false,
        };

        if is_allowed {
            return html! {
                for ctx.props().children.iter()
            };
        };

        match ctx.props().action {
            ForbiddenAction::None => html! {},
            ForbiddenAction::NotFound => html! {
                <NotFound />
            },
        }
    }
}
