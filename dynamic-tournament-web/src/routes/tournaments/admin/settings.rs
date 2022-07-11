use std::rc::Rc;

use chrono::{DateTime, NaiveDateTime, Utc};
use dynamic_tournament_api::v3::tournaments::{PartialTournament, Tournament};
use yew::{html, Component, Context, Html, Properties};

use crate::components::providers::{ClientProvider, Provider};
use crate::components::Input;
use crate::services::errorlog::ErrorLog;

#[derive(Clone, Debug, Properties)]
pub(super) struct Props {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

/// General tournament settings including the values of [`Tournament`].
#[derive(Debug)]
pub(super) struct Settings {
    tournament: PartialTournament,
}

impl Component for Settings {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            tournament: PartialTournament::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateName(name) => {
                self.tournament.name = Some(name);
            }
            Message::UpdateDate(date) => {
                self.tournament.date = Some(date);
            }
            Message::UpdateTournament => {
                let tournament = self.tournament.clone();
                let id = ctx.props().tournament.id;

                let client = ClientProvider::get(ctx);
                ctx.link().send_future_batch(async move {
                    match client.v3().tournaments().patch(id, &tournament).await {
                        Ok(_) => ErrorLog::info("Updated tournament"),
                        Err(err) => {
                            ErrorLog::error(format!("Failed to upate tournament: {:?}", err));
                        }
                    }

                    vec![]
                });
            }
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name = ctx.props().tournament.name.clone();
        let date = ctx.props().tournament.date.to_rfc3339();
        let kind = ctx.props().tournament.kind.to_string();

        let on_change_name = ctx.link().callback(Message::UpdateName);
        let on_change_date = ctx.link().callback(|date: String| {
            let date = match DateTime::parse_from_rfc3339(&date) {
                Ok(date) => date.with_timezone(&Utc),
                Err(err) => {
                    log::debug!("Failed to parse DateTime<FixedOffset>: {:?}", err);
                    DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
                }
            };

            Message::UpdateDate(date)
        });
        let on_update = ctx.link().callback(|_| Message::UpdateTournament);

        html! {
            <div>
                <h2>{ "Settings" }</h2>
                <div>
                    <span>{ "Name" }</span>
                    <Input kind={"text"} value={name} onchange={on_change_name} />
                </div>

                <div>
                    <span>{ "Date" }</span>
                    <Input kind={"text"} value={date} onchange={on_change_date} />
                </div>

                <div>
                    <span>{ "Entrant Type" }</span>
                    <input class="dt-input" type="text" value={ kind } disabled={ true } />
                </div>

                <button class="button" onclick={on_update}>{ "Update" }</button>
            </div>
        }
    }
}

#[allow(clippy::enum_variant_names)]
pub enum Message {
    UpdateName(String),
    UpdateDate(DateTime<Utc>),
    UpdateTournament,
}
