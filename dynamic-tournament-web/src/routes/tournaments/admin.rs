mod entrants;
mod settings;

use std::rc::Rc;

use dynamic_tournament_api::{v3::tournaments::Tournament, Client, Error};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::services::errorlog::ErrorLog;

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

pub struct Admin {}

impl Component for Admin {
    type Message = Message;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let tournament_id = ctx.props().tournament.id;

        match msg {
            Message::DeleteTournament => {
                let (client, _) = ctx
                    .link()
                    .context::<Client>(Callback::noop())
                    .expect("no client in context");

                ctx.link().send_future(async move {
                    Message::DeleteTournamentResult(
                        client.v3().tournaments().delete(tournament_id).await,
                    )
                });
            }
            Message::DeleteTournamentResult(result) => match result {
                Ok(()) => ErrorLog::info("Tournament deleted"),
                Err(err) => ErrorLog::error(format!("Failed to delete tournament: {}", err)),
            },
        }

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let delete_tournament = ctx.link().callback(|_| Message::DeleteTournament);

        html! {
            <div>
                <settings::Settings tournament={ctx.props().tournament.clone()} />
                <entrants::Entrants tournament={ctx.props().tournament.clone()} />
                <div>
                    <h2>{ "The Danger Zone" }</h2>

                    <button class="button">{ "Reset Tournament" }</button>
                    <button class="button" onclick={delete_tournament}>{ "Delete Tournament" }</button>
                </div>
            </div>
        }
    }
}

pub enum Message {
    DeleteTournament,
    DeleteTournamentResult(Result<(), Error>),
}
