use dynamic_tournament_api::tournament::Team;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use std::mem::{self, MaybeUninit};

use dynamic_tournament_generator::{EntrantScore, EntrantSpot};

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
        let link = ctx.link();

        // SAFETY: inputs is never read from before the value is properly initialized.
        let mut inputs: [Html; 2] = unsafe { std::mem::zeroed() };

        for (i, inp) in inputs.iter_mut().enumerate() {
            let on_score_update = {
                let link = link.clone();
                Callback::from(move |event: InputEvent| {
                    let input: HtmlInputElement = event.target_unchecked_into();
                    let value = input.value_as_number() as u64;

                    link.send_message(Msg::UpdateScore(i, value))
                })
            };

            let value = self.nodes[i].score;

            let team = match ctx.props().teams[i].clone() {
                EntrantSpot::Entrant(e) => e.name,
                // should be unreachable
                _ => "BYE".to_owned(),
            };

            *inp = html! {
                <div class="popup-team">
                    <span>{ team }</span>
                    <br />
                    <input type="number" value={value.to_string()} oninput={on_score_update} />
                </div>
            };
        }

        // SAFETY: `MaybeUninit` does not require any initialization.
        let mut teams: [MaybeUninit<Html>; 2] = unsafe { MaybeUninit::uninit().assume_init() };

        let mut winner_input: [MaybeUninit<Html>; 2] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for ((i, inp), winner_input) in teams.iter_mut().enumerate().zip(winner_input.iter_mut()) {
            let on_score_update = {
                let link = link.clone();
                Callback::from(move |event: InputEvent| {
                    let input: HtmlInputElement = event.target_unchecked_into();
                    let value = input.value_as_number() as u64;

                    link.send_message(Msg::UpdateScore(i, value))
                })
            };

            let on_winner_update = link.callback(move |_| Msg::UpdateWinner { index: i });

            let value = self.nodes[i].score;
            let winner = self.nodes[i].winner;

            let team = match ctx.props().teams[i].clone() {
                EntrantSpot::Entrant(e) => e.name,
                // should be unreachable
                _ => "BYE".to_owned(),
            };

            inp.write(html! {
                <tr>
                    <td>{ team.clone() }</td>
                    <td>
                        <input class="input-u64" type="number" min="0" value={value.to_string()} oninput={on_score_update}/>
                    </td>
                </tr>
            });

            let classes = if winner {
                "winner-input active"
            } else {
                "winner-input"
            };

            winner_input.write(html! {
                <button class={classes} onclick={on_winner_update}>{team}</button>
            });
        }

        // SAFETY: All items in `teams` are initialized.
        let teams: [Html; 2] = unsafe { mem::transmute(teams) };
        let winner_input: [Html; 2] = unsafe { mem::transmute(winner_input) };

        let on_submit = link.callback(|_| Msg::Submit);

        html! {
            <div class="flex-col2">
                <table class="table-striped">
                    <tr>
                        <th>{ "Team" }</th>
                        <th>{ "Score" }</th>
                    </tr>
                    { for teams.into_iter() }
                </table>
                <div class="winner-input-box">
                    <h3 class="h-center">{ "Declare a winner (optional)"}</h3>
                    <div class="flex-center winner-input-wrapper">
                        { for winner_input.into_iter() }
                    </div>
                </div>
                <button class="button" type="submit" onclick={on_submit} disabled=false>{ "Submit" }</button>
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub teams: [EntrantSpot<Team>; 2],
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
