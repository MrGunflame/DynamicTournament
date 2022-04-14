use yew::context::ContextProvider;
use yew::prelude::*;

use serde::{Deserialize, Serialize};

use std::rc::Rc;
use std::sync::Mutex;

use super::Provider;

pub struct AuthProvider {
    context: Auth,
}

impl Component for AuthProvider {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            context: Auth {
                inner: Rc::new(Mutex::new(None)),
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ContextProvider<Auth> context={self.context.clone()}>
                { for ctx.props().children.iter() }
            </ContextProvider<Auth>>
        }
    }
}

impl<C> Provider<Auth, C> for AuthProvider
where
    C: Component,
{
    fn take(ctx: &Context<C>) -> Auth {
        let (auth, _) = ctx
            .link()
            .context(Callback::noop())
            .expect("No AuthProvider given");

        auth
    }
}

/// Authentification data
#[derive(Clone, Debug)]
pub struct Auth {
    pub inner: Rc<Mutex<Option<InnerAuth>>>,
}

impl PartialEq for Auth {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InnerAuth {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Properties {
    pub children: Children,
}
