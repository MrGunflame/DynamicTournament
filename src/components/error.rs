use yew::prelude::*;

pub struct Error {
    error: String,
}

impl Component for Error {
    type Message = ();
    type Properties = ErrorProperties;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            error: ctx.props().error.clone(),
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <span>{"sadge"}</span>
                <span>{ self.error.clone() }</span>
            </>
        }
    }
}

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct ErrorProperties {
    pub error: String,
}
