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

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let text = if ctx.props().is_live {
            "Live"
        } else {
            "Disconnected"
        };

        let dot = if ctx.props().is_live {
            "dt-bracket-live-active"
        } else {
            "dt-bracket-live-inactive"
        };

        html! {
            <div class="dt-bracket-live">
                <span class={dot}></span>
                <span class="dt-bracket-live-label">{text}</span>
            </div>
        }
    }
}
