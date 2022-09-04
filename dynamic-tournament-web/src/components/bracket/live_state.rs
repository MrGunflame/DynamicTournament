use yew::{html, Component, Context, Html, Properties};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub is_live: bool,
}

#[derive(Debug)]
pub struct LiveState;

impl Component for LiveState {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div>
                { ctx.props().is_live.to_string() }
            </div>
        }
    }
}
