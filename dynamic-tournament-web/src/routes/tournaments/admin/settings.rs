use std::num::ParseIntError;
use std::str::FromStr;

use chrono::naive::{NaiveDate, NaiveTime};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dynamic_tournament_api::v3::tournaments::entrants::Entrant;
use dynamic_tournament_api::v3::tournaments::{EntrantKind, PartialTournament, Tournament};
use thiserror::Error;
use yew::{html, Component, Context, Html, Properties};

use crate::components::providers::{ClientProvider, Provider};
use crate::components::{Input, ParseInput};
use crate::services::errorlog::ErrorLog;
use crate::utils::{FetchData, Rc};

#[derive(Clone, Debug, PartialEq, Properties)]
pub(super) struct Props {
    pub tournament: Rc<Tournament>,
}

/// General tournament settings including the values of [`Tournament`].
#[derive(Debug)]
pub(super) struct Settings {
    datetime: DateTime<Local>,
    tournament: PartialTournament,

    entrants: FetchData<Vec<Entrant>>,
}

impl Component for Settings {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let id = ctx.props().tournament.id;
        ctx.link().send_future(async move {
            let msg = match client.v3().tournaments().entrants(id).list().await {
                Ok(entrants) => FetchData::new_with_value(entrants),
                Err(err) => FetchData::from_err(err),
            };

            Message::UpdateEntrants(msg)
        });

        Self {
            datetime: ctx.props().tournament.date.with_timezone(&Local),
            tournament: PartialTournament::default(),
            entrants: FetchData::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateEntrants(entrants) => {
                self.entrants = entrants;

                true
            }
            Message::UpdateName(name) => {
                self.tournament.name = Some(name);

                false
            }
            Message::UpdateDate(date) => {
                let time = self.datetime.time();

                let datetime = NaiveDateTime::new(date, time);

                self.datetime = Local.from_local_datetime(&datetime).unwrap();

                self.tournament.date = Some(self.datetime.with_timezone(&Utc));

                false
            }
            Message::UpdateTime(time) => {
                let date = self.datetime.date().naive_local();

                let datetime = NaiveDateTime::new(date, time);

                self.datetime = Local.from_local_datetime(&datetime).unwrap();

                self.tournament.date = Some(self.datetime.with_timezone(&Utc));

                false
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

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let name = ctx.props().tournament.name.clone();
        let kind = ctx.props().tournament.kind;

        let date = self.datetime.format("%d.%m.%Y").to_string();
        let time = self.datetime.format("%H:%M").to_string();

        let on_change_name = ctx.link().callback(Message::UpdateName);
        let on_change_date = ctx
            .link()
            .callback(|date: Date| Message::UpdateDate(date.0));

        let on_change_time = ctx
            .link()
            .callback(|time: Time| Message::UpdateTime(time.0));

        let on_update = ctx.link().callback(|_| Message::UpdateTournament);

        let mut kind_player = false;
        let mut kind_team = false;
        if kind == EntrantKind::Player {
            kind_player = true;
        } else {
            kind_team = true;
        }

        let disabled = !self
            .entrants
            .as_ref()
            .map(|entrants| entrants.is_empty())
            .unwrap_or_default();

        html! {
            <div>
                <h2>{ "Settings" }</h2>
                <div>
                    <table class="dt-table-striped">
                        <tr>
                            <td>
                                { "Name" }
                            </td>
                            <td>
                                <Input kind={"text"} value={name} onchange={on_change_name} />
                            </td>
                        </tr>
                        <tr>
                            <td>
                                { "Date" }
                            </td>
                            <td>
                                <ParseInput<Date> value={date} onchange={on_change_date} />
                            </td>
                        </tr>
                        <tr>
                            <td>
                                { "Time" }
                            </td>
                            <td>
                                <ParseInput<Time> value={time} onchange={on_change_time} />
                            </td>
                        </tr>
                        <tr>
                            <td>
                                { "Type" }
                            </td>
                            <td>
                                <select {disabled}>
                                    <option selected={kind_player}>{ "Player" }</option>
                                    <option selected={kind_team}>{ "Team" }</option>
                                </select>
                            </td>
                        </tr>
                    </table>
                </div>

                <button class="dt-button" onclick={on_update}>{ "Update" }</button>
            </div>
        }
    }
}

#[allow(clippy::enum_variant_names)]
pub enum Message {
    UpdateEntrants(FetchData<Vec<Entrant>>),
    UpdateName(String),
    UpdateDate(NaiveDate),
    UpdateTime(NaiveTime),
    UpdateTournament,
}

struct Date(pub NaiveDate);

impl FromStr for Date {
    type Err = ParseDateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');

        let day = parts.next().ok_or(ParseDateError::InvalidParts(1))?;
        let month = parts.next().ok_or(ParseDateError::InvalidParts(2))?;
        let year = parts.next().ok_or(ParseDateError::InvalidParts(3))?;

        let c = parts.count();
        if c > 0 {
            return Err(ParseDateError::InvalidParts(c + 3));
        }

        let day: u32 = day.parse()?;
        let month: u32 = month.parse()?;
        let year: i32 = year.parse()?;

        let date = NaiveDate::from_ymd_opt(year, month, day).ok_or(ParseDateError::InvalidDate)?;

        Ok(Self(date))
    }
}

#[derive(Debug, Error)]
enum ParseDateError {
    #[error("invalid number of parts: expected 3, found {0}")]
    InvalidParts(usize),
    #[error("failed to parse value: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("invalid date")]
    InvalidDate,
}

struct Time(pub NaiveTime);

impl FromStr for Time {
    type Err = ParseTimeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');

        let hour = parts.next().ok_or(ParseTimeError::InvalidParts(1))?;
        let minute = parts.next().ok_or(ParseTimeError::InvalidParts(2))?;

        let c = parts.count();
        if c > 0 {
            return Err(ParseTimeError::InvalidParts(c + 2));
        }

        let hour = hour.parse()?;
        let minute = minute.parse()?;

        let time = NaiveTime::from_hms_opt(hour, minute, 0).ok_or(ParseTimeError::InvalidTime)?;

        Ok(Self(time))
    }
}

#[derive(Debug, Error)]
enum ParseTimeError {
    #[error("invalid number of parts: expected 2, found {0}")]
    InvalidParts(usize),
    #[error("failed to parse value: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("invalid time")]
    InvalidTime,
}
