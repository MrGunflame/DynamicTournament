use std::rc::Rc;

use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::Bracket as ApiBracket;
use dynamic_tournament_api::Client;
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::utils::FetchData;

use crate::components::admin::AdminBracket;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament_id: TournamentId,
    pub id: BracketId,
}

#[derive(Debug)]
pub struct Bracket {
    bracket: FetchData<ApiBracket>,
}

impl Component for Bracket {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let tournament_id = ctx.props().tournament_id;
        let id = ctx.props().id;

        let (client, _) = ctx.link().context::<Client>(Callback::noop()).unwrap();
        ctx.link().send_future(async move {
            match client
                .v3()
                .tournaments()
                .brackets(tournament_id)
                .get(id)
                .await
            {
                Ok(bracket) => Message::UpdateBracket(FetchData::from(bracket)),
                Err(err) => Message::UpdateBracket(FetchData::from_err(err)),
            }
        });

        Self {
            bracket: FetchData::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateBracket(bracket) => {
                self.bracket = bracket;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.bracket.render(|bracket| {
            let bracket = Rc::new(bracket.clone());

            html! {
                <div>
                    <h2>{"pog"}</h2>
                    <AdminBracket {bracket} />
                </div>
            }
        })
    }
}

pub enum Message {
    UpdateBracket(FetchData<ApiBracket>),
}
