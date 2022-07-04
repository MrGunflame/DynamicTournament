use std::rc::Rc;

use dynamic_tournament_api::{
    v3::{
        id::BracketId,
        tournaments::{brackets::BracketOverview, Tournament},
    },
    Client,
};
use yew::{html, Callback, Component, Context, Html, Properties};
use yew_router::{history::History, prelude::RouterScopeExt};

use crate::utils::FetchData;

use super::Route;

pub mod bracket;

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

#[derive(Debug)]
pub struct Brackets {
    brackets: FetchData<Vec<BracketOverview>>,
}

impl Component for Brackets {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let (client, _) = ctx
            .link()
            .context::<Client>(Callback::noop())
            .expect("no client in context");

        let tournament_id = ctx.props().tournament.id;
        ctx.link().send_future(async move {
            let msg = match client
                .v3()
                .tournaments()
                .brackets(tournament_id)
                .list()
                .await
            {
                Ok(brackets) => FetchData::from(brackets),
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
                    if brackets.len() == 1 {
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

                ctx.link()
                    .history()
                    .expect("no history in context")
                    .push(Route::Bracket {
                        tournament_id,
                        tournament_name,
                        bracket_id: id,
                        bracket_name: name,
                    });
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

            let brackets: Html = brackets
                .iter()
                .map(|bracket| {
                    let id = bracket.id;
                    let name = bracket.name.clone();

                    let onclick = ctx
                        .link()
                        .callback(move |_| Message::OnClick(id, name.clone()));

                    html! {
                        <tr {onclick} class="tr-link">
                            <td>{ bracket.name.clone() }</td>
                        </tr>
                    }
                })
                .collect();

            html! {
                <div>
                    <table class="table-center">
                        <tbody>
                            <tr>
                                <th>{ "Name" }</th>
                            </tr>
                            { brackets }
                        </tbody>
                    </table>
                </div>
            }
        })
    }
}

pub enum Message {
    Update(FetchData<Vec<BracketOverview>>),
    OnClick(BracketId, String),
}
