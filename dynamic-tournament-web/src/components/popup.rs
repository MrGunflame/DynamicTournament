use web_sys::{Element, KeyboardEvent};
use yew::create_portal;
use yew::prelude::*;

pub struct Popup {
    host: Element,
}

impl Component for Popup {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();

        let host = document.get_element_by_id("popup-host").unwrap();

        Self { host }
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

        create_portal(
            html! {
                <div tabindex="-1" class="popup-wrapper" onkeydown={onkeydown}>
                    <div class="popup">
                        <div class="popup-close-wrapper">
                            <button class="popup-close" onclick={on_close} title="Close" disabled=false>
                                <i class="fa-xmark fa-solid fa-2xl"></i>
                            </button>
                        </div>
                        <div class="popup-content">
                            { for ctx.props().children.iter() }
                        </div>
                    </div>
                </div>
            },
            self.host.clone(),
        )
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
