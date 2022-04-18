use yew::prelude::*;

pub struct Loader;

impl Component for Loader {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <span>{"Loading.."}</span>
        }
    }
}
