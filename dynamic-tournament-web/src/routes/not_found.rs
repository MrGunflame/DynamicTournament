use yew::prelude::*;

/// A 404 error handler.
pub struct NotFound;

impl Component for NotFound {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div>
                <h1>{ "404" }</h1>
                <span>{ "This page doesn't exist. sadge" }</span>
            </div>
        }
    }
}
