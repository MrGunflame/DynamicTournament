use dynamic_tournament_api::v3::{id::BracketId, tournaments::brackets::BracketOverview};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::utils::Rc;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub brackets: Rc<Vec<BracketOverview>>,
    pub onclick: Callback<(usize, BracketId)>,
    #[prop_or_default]
    pub active_bracket: BracketId,
}

#[derive(Debug)]
pub struct BracketList {
    active: usize,
}

impl Component for BracketList {
    type Message = (usize, BracketId);
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        if ctx.props().active_bracket != 0 {
            for (index, bracket) in ctx.props().brackets.iter().enumerate() {
                if bracket.id == ctx.props().active_bracket {
                    return Self { active: index };
                }
            }
        }

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
                    "dt-brlist-bracket dt-active"
                } else {
                    "dt-brlist-bracket"
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
