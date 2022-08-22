use dynamic_tournament_api::v3::tournaments::brackets::BracketOverview;
use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::{html, Component, Context, Html, Properties};

use super::Route;
use crate::components::providers::{ClientProvider, Provider};
use crate::utils::router::{Redirect, RouterContextExt};
use crate::utils::{FetchData, Rc};

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

#[derive(Debug)]
pub struct Brackets {
    brackets: FetchData<Vec<BracketOverview>>,
}

impl Component for Brackets {
    type Message = FetchData<Vec<BracketOverview>>;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let tournament_id = ctx.props().tournament.id;
        let client = ClientProvider::get(ctx);

        ctx.link().send_future(async move {
            match client
                .v3()
                .tournaments()
                .brackets(tournament_id)
                .list()
                .await
            {
                Ok(val) => FetchData::from(val),
                Err(err) => FetchData::from_err(err),
            }
        });

        Self {
            brackets: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: FetchData<Vec<BracketOverview>>) -> bool {
        self.brackets = msg;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        self.brackets.render(|brackets| match brackets.first() {
            Some(bracket) => {
                let to = format!(
                    "/tournaments/{}/brackets/{}",
                    ctx.props().tournament.id,
                    bracket.id
                );

                html! {
                    <Redirect {to} />
                }
            }
            None => html! {
                <span>{ "No brackets avaliable yet." }</span>
            },
        })
    }
}
