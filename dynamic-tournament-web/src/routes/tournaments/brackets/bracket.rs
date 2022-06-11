use std::rc::Rc;

use dynamic_tournament_api::v3::tournaments::entrants::Entrant;
use dynamic_tournament_api::v3::tournaments::Tournament;
use dynamic_tournament_api::v3::{id::BracketId, tournaments::brackets::Bracket as ApiBracket};
use dynamic_tournament_api::Client;
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::components::bracket::Bracket as BracketComponent;
use crate::components::movable_boxed::MovableBoxed;
use crate::utils::FetchData;

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
    pub id: BracketId,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

pub struct Bracket {
    bracket: FetchData<Rc<ApiBracket>>,
    entrants: FetchData<Rc<Vec<Entrant>>>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("no client in context");

        let tournament_id = ctx.props().tournament.id;
        let id = ctx.props().id;
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
                    Ok(bracket) => FetchData::from(Rc::new(bracket)),
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
            bracket: FetchData::new(),
            entrants: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateBracket(bracket) => self.bracket = bracket,
            Message::UpdateEntrants(entrants) => self.entrants = entrants,
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.bracket.has_value() && self.entrants.has_value() {
            let tournament = ctx.props().tournament.clone();
            let bracket = self.bracket.clone().unwrap();
            let entrants = self.entrants.clone().unwrap();

            html! {
                <MovableBoxed>
                    <BracketComponent {tournament} {bracket} {entrants} />
                </MovableBoxed>
            }
        } else {
            html! {
                <span>{"Loading"}</span>
            }
        }
    }
}

pub enum Message {
    UpdateBracket(FetchData<Rc<ApiBracket>>),
    UpdateEntrants(FetchData<Rc<Vec<Entrant>>>),
}
