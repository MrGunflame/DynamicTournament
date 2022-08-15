use dynamic_tournament_api::v3::{
    id::BracketId,
    tournaments::{brackets::BracketOverview, Tournament},
};
use yew::{html, Component, Context, Html, Properties};

use crate::utils::router::RouterContextExt;
use crate::utils::FetchData;
use crate::{
    components::{
        providers::{ClientProvider, Provider},
        BracketList,
    },
    utils::Rc,
};

use super::Route;

pub mod bracket;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

#[derive(Debug)]
pub struct Brackets {
    brackets: FetchData<Rc<Vec<BracketOverview>>>,
}

impl Component for Brackets {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let tournament_id = ctx.props().tournament.id;
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

            Message::Update(msg)
        });

        Self {
            brackets: FetchData::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::Update(brackets) => {
                self.brackets = brackets;

                // Redirect to the bracket when there's only one avaliable.
                self.brackets.as_ref().map(|brackets| {
                    if !brackets.is_empty() {
                        let bracket = &brackets[0];

                        log::debug!("Redirecting to bracket {}", bracket.id);

                        ctx.link()
                            .send_message(Message::OnClick(bracket.id, bracket.name.clone()));
                    }
                });
            }
            Message::OnClick(id, name) => {
                let tournament_id = ctx.props().tournament.id;
                let tournament_name = ctx.props().tournament.name.clone();

                ctx.history().redirect(Route::Bracket { id, name });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.brackets.render(|brackets| {
            if brackets.is_empty() {
                return html! {
                    <div>{ "There are currently no brackets avaliable." }</div>
                };
            }

            let brackets = brackets.clone();
            let onclick = ctx
                .link()
                .callback(move |(_, id)| Message::OnClick(id, "a".into()));

            html! {
                <BracketList {brackets} {onclick} />
            }
        })
    }
}

pub enum Message {
    Update(FetchData<Rc<Vec<BracketOverview>>>),
    OnClick(BracketId, String),
}
