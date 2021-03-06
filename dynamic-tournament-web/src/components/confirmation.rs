use yew::prelude::*;

use crate::components::popup::Popup;

pub struct Confirmation;

impl Component for Confirmation {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Confirm => ctx.props().on_confirm.emit(()),
            Message::Cancel => ctx.props().on_close.emit(()),
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_confirm = ctx.link().callback(|_| Message::Confirm);
        let on_cancel = ctx.link().callback(|_| Message::Cancel);

        let on_close = ctx.link().callback(|_| Message::Cancel);

        html! {
            <Popup on_close={on_close}>
                { for ctx.props().children.iter() }
                <div>
                    <button class="button" onclick={on_confirm}>{ "Yes" }</button>
                    <button class="button" onclick={on_cancel}>{ "Cancel" }</button>
                </div>
            </Popup>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
    pub on_close: Callback<()>,
    pub on_confirm: Callback<()>,
}

pub enum Message {
    Confirm,
    Cancel,
}
