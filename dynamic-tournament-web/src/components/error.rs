use yew::prelude::*;

pub struct Error;

impl Component for Error {
    type Message = ();
    type Properties = ErrorProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <span>{"sadge"}</span>
                <span>{ ctx.props().error.clone() }</span>
            </>
        }
    }
}

#[derive(Clone, Debug, Properties, PartialEq, Eq)]
pub struct ErrorProperties {
    pub error: String,
}
