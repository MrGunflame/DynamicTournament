use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{Element, KeyboardEvent};
use yew::create_portal;
use yew::prelude::*;

use crate::components::icons::{FaSize, FaXmark};
use crate::utils::document;

pub struct Popup {
    host: Element,
    listener: Option<EventListener>,
}

impl Component for Popup {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        let document = web_sys::window().unwrap().document().unwrap();

        let host = document.get_element_by_id("popup-host").unwrap();

        Self {
            host,
            listener: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Close => {
                ctx.props().on_close.emit(());
                false
            }
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }

        let onkeydown =
            ctx.link()
                .batch_callback(move |event: KeyboardEvent| match event.code().as_str() {
                    "Escape" => Some(Message::Close),
                    _ => None,
                });

        let keydown = EventListener::new(&document(), "keydown", move |event| {
            onkeydown.emit(event.dyn_ref::<KeyboardEvent>().unwrap().clone());
        });

        self.listener = Some(keydown);
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_close = ctx.link().callback(|_| Message::Close);

        create_portal(
            html! {
                <div tabindex="-1" class="popup-wrapper">
                    <div class="popup">
                        <div class="popup-close-wrapper">
                            <button class="popup-close" onclick={on_close} title="Close" disabled=false>
                                <FaXmark label="Close" size={FaSize::ExtraLarge2} />
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
