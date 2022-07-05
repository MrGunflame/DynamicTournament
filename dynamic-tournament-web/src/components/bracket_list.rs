use std::rc::Rc;

use dynamic_tournament_api::v3::{id::BracketId, tournaments::brackets::BracketOverview};
use yew::{html, Callback, Component, Context, Html, Properties};

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub brackets: Rc<Vec<BracketOverview>>,
    pub onclick: Callback<(usize, BracketId)>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.brackets, &other.brackets)
    }
}

#[derive(Debug)]
pub struct BracketList {
    active: usize,
}

impl Component for BracketList {
    type Message = (usize, BracketId);
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { active: 0 }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.active = msg.0;
        ctx.props().onclick.emit(msg);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let brackets: Html = ctx
            .props()
            .brackets
            .iter()
            .enumerate()
            .map(|(index, bracket)| {
                let id = bracket.id;
                let onclick = ctx.link().callback(move |_| (index, id));

                let classes = if self.active == index {
                    "dt-r-bracket active"
                } else {
                    "dt-r-bracket"
                };

                html! {
                    <span {onclick} class={classes}>{ bracket.name.clone() }</span>
                }
            })
            .collect();

        html! {
            <div>
                { brackets }
            </div>
        }
    }
}
