use web_sys::KeyboardEvent;
use yew::prelude::*;

pub struct Popup;

impl Component for Popup {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Close => {
                ctx.props().on_close.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_close = ctx.link().callback(|_| Message::Close);

        // Close the popup with the Escape key.
        let onkeydown = ctx
            .link()
            .batch_callback(|e: KeyboardEvent| match e.code().as_str() {
                "Escape" => Some(Message::Close),
                _ => None,
            });

        html! {
            <div tabindex="-1" class="popup-wrapper" onkeydown={onkeydown}>
                <div class="popup">
                    <div class="popup-close-wrapper">
                        <button class="popup-close" onclick={on_close} title="Close Popup" disabled=false>
                            <img src="/assets/xmark-solid.svg" width="32px" height="32px" alt="x" />
                        </button>
                    </div>
                    <div class="popup-content">
                        { for ctx.props().children.iter() }
                    </div>
                </div>
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
    pub on_close: Callback<()>,
}

pub enum Message {
    Close,
}
