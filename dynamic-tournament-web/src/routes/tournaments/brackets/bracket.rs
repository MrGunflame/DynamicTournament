use dynamic_tournament_api::v3::tournaments::brackets::BracketOverview;
use dynamic_tournament_api::v3::tournaments::entrants::Entrant;
use dynamic_tournament_api::v3::tournaments::Tournament;
use dynamic_tournament_api::v3::{id::BracketId, tournaments::brackets::Bracket as ApiBracket};
use yew::{html, Component, Context, Html, Properties};

use super::Route;
use crate::components::bracket::Bracket as BracketComponent;
use crate::components::movable_boxed::MovableBoxed;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::BracketList;
use crate::utils::router::RouterContextExt;
use crate::utils::{FetchData, Rc};

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
    pub bracket_id: BracketId,
}

pub struct Bracket {
    brackets: FetchData<Rc<Vec<BracketOverview>>>,
    bracket: FetchData<Rc<ApiBracket>>,
    entrants: FetchData<Rc<Vec<Entrant>>>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let tournament_id = ctx.props().tournament.id;
        let id = ctx.props().bracket_id;
        {
            let client = client.clone();

            ctx.link().send_future(async move {
                let msg = match client
                    .v3()
                    .tournaments()
                    .brackets(tournament_id)
                    .list()
                    .await
                {
                    Ok(brackets) => FetchData::from(Rc::new(brackets)),
                    Err(err) => FetchData::from_err(err),
                };

                Message::UpdateBrackets(msg)
            });
        }

        {
            let client = client.clone();
            ctx.link().send_future(async move {
                let msg = match client
                    .v3()
                    .tournaments()
                    .brackets(tournament_id)
                    .get(id)
                    .await
                {
                    Ok(b) => FetchData::from(Rc::new(b)),
                    Err(err) => FetchData::from_err(err),
                };

                Message::UpdateBracket(msg)
            });
        }

        ctx.link().send_future(async move {
            let msg = match client
                .v3()
                .tournaments()
                .entrants(tournament_id)
                .list()
                .await
            {
                Ok(entrants) => FetchData::from(Rc::new(entrants)),
                Err(err) => FetchData::from_err(err),
            };

            Message::UpdateEntrants(msg)
        });

        Self {
            brackets: FetchData::new(),
            entrants: FetchData::new(),
            bracket: FetchData::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateBrackets(bracket) => self.brackets = bracket,
            Message::UpdateEntrants(entrants) => self.entrants = entrants,
            Message::UpdateBracket(bracket) => self.bracket = bracket,
            Message::OnClick(id, name) => {
                let tournament_id = ctx.props().tournament.id;

                // Don't update when requesting the same bracket.
                if self.bracket.has_value() && self.bracket.as_ref().unwrap().id == id {
                    return false;
                }

                let client = ClientProvider::get(ctx);

                ctx.link().send_future(async move {
                    let msg = match client
                        .v3()
                        .tournaments()
                        .brackets(tournament_id)
                        .get(id)
                        .await
                    {
                        Ok(b) => FetchData::from(Rc::new(b)),
                        Err(err) => FetchData::from_err(err),
                    };

                    Message::UpdateBracket(msg)
                });

                ctx.history().redirect(Route::Bracket { id });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.brackets.has_value() && self.entrants.has_value() && self.bracket.has_value() {
            let tournament = ctx.props().tournament.clone();
            let brackets = self.brackets.clone().unwrap();
            let entrants = self.entrants.clone().unwrap();
            let bracket = self.bracket.clone().unwrap();

            let onclick = ctx
                .link()
                .callback(move |(_, id)| Message::OnClick(id, "a".into()));

            html! {
                <>
                    <BracketList {brackets} {onclick} />
                    <MovableBoxed>
                        <BracketComponent {tournament} {bracket} {entrants} />
                    </MovableBoxed>
                </>
            }
        } else {
            html! {
                <span>{"Loading"}</span>
            }
        }
    }
}

pub enum Message {
    UpdateBrackets(FetchData<Rc<Vec<BracketOverview>>>),
    UpdateBracket(FetchData<Rc<ApiBracket>>),
    UpdateEntrants(FetchData<Rc<Vec<Entrant>>>),
    OnClick(BracketId, String),
}
