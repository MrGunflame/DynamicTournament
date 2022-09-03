use dynamic_tournament_core::{EntrantScore, EntrantSpot};
use yew::{html, Callback, Component, Context, Html, Properties};

use crate::components::ParseInput;

pub struct BracketUpdate {
    // Score: [left, right]
    nodes: [EntrantScore<u64>; 2],
}

impl Component for BracketUpdate {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            nodes: ctx.props().nodes,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateScore(index, score) => {
                self.nodes[index].score = score;
                true
            }
            Msg::UpdateWinner { index } => {
                let value = !self.nodes[index].winner;

                // Make sure we only have 1 winner.
                if value {
                    for (i, node) in self.nodes.iter_mut().enumerate() {
                        if i != index {
                            node.winner = false;
                        }
                    }
                }

                self.nodes[index].winner = value;

                true
            }
            Msg::Submit => {
                ctx.props().on_submit.emit(self.nodes);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut entrants: [Html; 2] = [html! {}, html! {}];
        let mut winners: [Html; 2] = [html! {}, html! {}];

        for (index, entrant) in entrants.iter_mut().enumerate() {
            let onchange = ctx
                .link()
                .callback(move |value| Msg::UpdateScore(index, value));

            let score = self.nodes[index].score.to_string();
            let winner = self.nodes[index].winner;

            let team = match ctx.props().teams[index].clone() {
                EntrantSpot::Entrant(entrant) => entrant,
                _ => "BYE".to_owned(),
            };

            *entrant = html! {
                <tr class="popup-team">
                    <td>{ team.clone() }</td>
                    <td><ParseInput<u64> classes="input-u64" kind="number" value={score} {onchange} /></td>
                </tr>
            };

            let classes = if winner {
                "winner-input active"
            } else {
                "winner-input"
            };

            let onclick = ctx.link().callback(move |_| Msg::UpdateWinner { index });

            winners[index] = html! {
                <button class={classes} {onclick}>{ team }</button>
            };
        }

        let on_submit = ctx.link().callback(|_| Msg::Submit);

        html! {
            <div class="flex-col2">
                <table class="table-striped">
                    <tr>
                        <th>{ "Team" }</th>
                        <th>{ "Score" }</th>
                    </tr>
                    { for entrants.into_iter() }
                </table>
                <div class="winner-input-box">
                    <h3 class="h-center">{ "Declare a winner (optional)"}</h3>
                    <div class="flex-center winner-input-wrapper">
                        { for winners.into_iter() }
                    </div>
                </div>
                <button class="button" type="submit" onclick={on_submit} disabled=false>{ "Submit" }</button>
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub teams: [EntrantSpot<String>; 2],
    pub nodes: [EntrantScore<u64>; 2],
    pub on_submit: Callback<[EntrantScore<u64>; 2]>,
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.on_submit == other.on_submit && self.nodes == other.nodes
    }
}

pub enum Msg {
    UpdateScore(usize, u64),
    UpdateWinner { index: usize },
    Submit,
}
