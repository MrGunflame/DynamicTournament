use web_sys::KeyboardEvent;
use yew::prelude::*;

/// An interactive button.
pub struct Button;

impl Component for Button {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        match &ctx.props().onclick {
            Some(cb) => cb.emit(()),
            None => (),
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let title = ctx.props().title;
        let classes = ctx.props().classes;

        if ctx.props().disabled {
            html! {
                <button role="button" class={classes} {title} disabled=true>
                    { for ctx.props().children.iter() }
                </button>
            }
        } else {
            let onclick = ctx.link().callback(move |_| ());

            let onkeydown =
                ctx.link()
                    .batch_callback(move |e: KeyboardEvent| match e.code().as_str() {
                        "Return" => Some(()),
                        _ => None,
                    });

            html! {
                <button role="button" class={classes} {title} {onclick} {onkeydown} tabindex=0>
                    { for ctx.props().children.iter() }
                </button>
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Properties {
    pub children: Children,
    #[prop_or_default]
    pub onclick: Option<Callback<()>>,
    #[prop_or_default]
    pub disabled: bool,
    pub title: &'static str,
    #[prop_or("button")]
    pub classes: &'static str,
}
