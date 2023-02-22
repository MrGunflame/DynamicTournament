use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::{html, Component, Context, Html, Properties};

use crate::components::confirmation::Confirmation;
use crate::components::Button;
use crate::utils::Rc;
use crate::{
    components::providers::{ClientProvider, Provider},
    services::MessageLog,
};

const DELETE_MESSAGE: &str = "Delete this tournament? This operation cannot be undone.";

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

#[derive(Debug)]
pub struct DangerZone {
    popup: Option<PopupState>,
}

impl Component for DangerZone {
    type Message = Message;
    type Properties = Props;

    #[inline]
    fn create(_ctx: &Context<Self>) -> Self {
        Self { popup: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::OpenPopup(msg) => {
                self.popup = Some(msg);

                true
            }
            Message::ClosePopup => {
                self.popup = None;

                true
            }
            Message::DeleteTournament => {
                let id = ctx.props().tournament.id;

                let client = ClientProvider::get(ctx);
                ctx.link().send_future_batch(async move {
                    match client.v3().tournaments().delete(id).await {
                        Ok(()) => MessageLog::info("Tournament deleted"),
                        Err(err) => MessageLog::error(err),
                    }

                    vec![]
                });

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let delete = ctx
            .link()
            .callback(|_| Message::OpenPopup(PopupState::DeleteTournament));

        let popup = match self.popup {
            Some(msg) => {
                // Cancel handler
                let close = ctx.link().callback(|_| Message::ClosePopup);

                // Confirmation handler
                let (msg, confirm) = match msg {
                    PopupState::DeleteTournament => {
                        let cb = ctx.link().callback(|_| Message::DeleteTournament);

                        (DELETE_MESSAGE, cb)
                    }
                };

                html! {
                    <Confirmation on_close={close} on_confirm={confirm}>
                        <span>{ msg }</span>
                    </Confirmation>
                }
            }
            None => html! {},
        };

        html! {
            <>
                {popup}

                <div>
                    <h2>{ "The Danger Zone" }</h2>

                    <Button classes="dt-button-red">{ "Reset Tournament" }</Button>
                    <Button classes="dt-button-red" onclick={delete}>{ "Delete Tournament" }</Button>
                </div>
            </>
        }
    }
}

#[derive(Debug)]
pub enum Message {
    OpenPopup(PopupState),
    ClosePopup,
    DeleteTournament,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum PopupState {
    DeleteTournament,
}
