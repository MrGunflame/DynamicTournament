pub mod bracket;

use std::rc::Rc;

use dynamic_tournament_api::{
    v3::{
        id::BracketId,
        tournaments::{brackets::BracketOverview, Tournament},
    },
    Client,
};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::utils::FetchData;

#[derive(Clone, Debug, Properties)]
pub(super) struct Props {
    pub tournament: Rc<Tournament>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.tournament, &other.tournament)
    }
}

#[derive(Debug)]
pub(super) struct Brackets {
    brackets: FetchData<Vec<BracketOverview>>,
}

impl Component for Brackets {
    type Message = Message;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().tournament.id;

        let (client, _) = ctx.link().context::<Client>(Callback::noop()).unwrap();
        ctx.link().send_future(async move {
            match client.v3().tournaments().brackets(id).list().await {
                Ok(brackets) => Message::UpdateBrackets(FetchData::from(brackets)),
                Err(err) => Message::UpdateBrackets(FetchData::from_err(err)),
            }
        });

        Self {
            brackets: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateBrackets(brackets) => {
                self.brackets = brackets;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.brackets.render(|brackets| {
            let body: Html = brackets
                .iter()
                .map(|bracket| {
                    html! {
                        <tr>
                            <td>{ bracket.name.clone() }</td>
                        </tr>
                    }
                })
                .collect();

            html! {
                <div>
                    <h2>{ "Brackets" }</h2>
                    <table>
                        <tr>
                            <th>{ "Name" }</th>
                        </tr>
                        { body }
                    </table>
                </div>
            }
        })
    }
}

pub enum Message {
    UpdateBrackets(FetchData<Vec<BracketOverview>>),
}
