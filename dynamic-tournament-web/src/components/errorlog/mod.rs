use yew::html;
use yew::html::{Component, Context, Html};
use yew_agent::{Bridge, Bridged};

use crate::services::errorlog::ErrorLogBus;

pub struct ErrorLog {
    log: Vec<String>,
    _producer: Box<dyn Bridge<ErrorLogBus>>,
}

impl Component for ErrorLog {
    type Properties = ();
    type Message = Message;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            log: Vec::new(),
            _producer: ErrorLogBus::bridge(ctx.link().callback(Message::Append)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Append(msg) => self.log.push(msg),
            Message::Remove(index) => {
                self.log.remove(index);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let msgs: Html = self
            .log
            .iter()
            .enumerate()
            .map(|(index, msg)| {
                let onclick = ctx.link().callback(move |_| Message::Remove(index));

                html! {
                    <div class="dt-msglog-msg">
                        <div class="dt-msglog-button">
                            <button {onclick}>
                                <i aria-hidden="true" class="fa-xmark fa-solid fa-2xl"></i>
                                <span class="sr-only">{ "Close" }</span>
                            </button>
                        </div>
                        <span>{ msg }</span>
                    </div>

                }
            })
            .collect();

        html! {
            <div class="dt-msglog">
                { msgs }
            </div>
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Append(String),
    Remove(usize),
}
