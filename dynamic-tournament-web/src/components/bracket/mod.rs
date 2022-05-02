pub mod double_elimination;
pub mod single_elimination;

mod r#match;
mod team;

use double_elimination::DoubleEliminationBracket;
use r#match::{Action, BracketMatch};
use single_elimination::SingleEliminationBracket;
use team::BracketTeam;

use dynamic_tournament_api::tournament::{self, BracketType, Tournament, TournamentId};
use dynamic_tournament_api::Client;

use std::rc::Rc;

use yew::prelude::*;

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::FetchData;

#[derive(Debug)]
pub struct Bracket {
    bracket: FetchData<Option<Rc<tournament::Bracket>>>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Properties;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::take(ctx);
        let id = ctx.props().tournament.id;

        ctx.link().send_future(async move {
            async fn fetch_data(
                client: Client,
                id: TournamentId,
            ) -> FetchData<Option<Rc<tournament::Bracket>>> {
                let client = client.tournaments();

                match client.bracket(id).get().await {
                    Ok(bracket) => FetchData::new_with_value(Some(Rc::new(bracket))),
                    Err(_) => FetchData::new_with_value(None),
                }
            }

            let data = fetch_data(client, id).await;

            Message::Update(data)
        });

        Self {
            bracket: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Update(data) => {
                self.bracket = data;

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.bracket.render(|data| {
            let tournament = ctx.props().tournament.clone();
            let bracket = data.clone();

            match ctx.props().tournament.bracket_type {
                BracketType::SingleElimination => html! {
                    <SingleEliminationBracket tournament={tournament} bracket={bracket} />
                },
                BracketType::DoubleElimination => html! {
                    <DoubleEliminationBracket tournament={tournament} bracket={bracket} />
                },
            }
        })
    }
}

pub enum Message {
    Update(FetchData<Option<Rc<tournament::Bracket>>>),
}

#[derive(Clone, Debug, Properties)]
pub struct Properties {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Properties {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}
