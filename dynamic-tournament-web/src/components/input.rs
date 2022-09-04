use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::{html, Callback, Classes, Component, Context, Html, Properties};

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub kind: &'static str,
    pub value: String,
    pub onchange: Callback<String>,
    #[prop_or_default]
    pub classes: Classes,
}

#[derive(Debug)]
pub struct Input {
    value: String,
}

impl Component for Input {
    type Message = String;
    type Properties = Props;

    #[inline]
    fn create(ctx: &Context<Self>) -> Self {
        Self {
            value: ctx.props().value.clone(),
        }
    }

    #[inline]
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.value = msg;
        ctx.props().onchange.emit(self.value.clone());
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let kind = ctx.props().kind;
        let value = ctx.props().value.clone();

        let onchange = ctx.link().callback(|event: Event| match event.target() {
            Some(target) => {
                let target: HtmlInputElement = target.dyn_into().unwrap();
                target.value()
            }
            None => unreachable!(),
        });

        let classes = match &ctx.props().classes.is_empty() {
            false => ctx.props().classes.clone(),
            true => Classes::from("dt-input"),
        };

        html! {
            <input class={classes} type={kind} {value} {onchange} />
        }
    }
}
