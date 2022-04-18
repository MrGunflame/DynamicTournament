use dynamic_tournament_api::tournament::Team;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use std::mem::{self, MaybeUninit};

use dynamic_tournament_generator::{EntrantSpot, EntrantWithScore};

pub struct BracketUpdate {
    // Score: [left, right]
    scores: [u64; 2],
}

impl Component for BracketUpdate {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            scores: ctx.props().scores,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateScore(index, score) => {
                self.scores[index] = score;
                true
            }
            Msg::Submit => {
                ctx.props().on_submit.emit(self.scores);
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

            let value = self.scores[i];

            let team = match ctx.props().teams[i].clone() {
                EntrantSpot::Entrant(e) => e.entrant.name,
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

        for (i, inp) in teams.iter_mut().enumerate() {
            let on_score_update = {
                let link = link.clone();
                Callback::from(move |event: InputEvent| {
                    let input: HtmlInputElement = event.target_unchecked_into();
                    let value = input.value_as_number() as u64;

                    link.send_message(Msg::UpdateScore(i, value))
                })
            };

            let value = self.scores[i];

            let team = match ctx.props().teams[i].clone() {
                EntrantSpot::Entrant(e) => e.entrant.name,
                // should be unreachable
                _ => "BYE".to_owned(),
            };

            inp.write(html! {
                <tr>
                    <td>{ team }</td>
                    <td>
                        <input class="input-u64" type="number" min="0" value={value.to_string()} oninput={on_score_update}/>
                    </td>
                </tr>
            });
        }

        // SAFETY: All items in `teams` are initialized.
        let teams: [Html; 2] = unsafe { mem::transmute(teams) };

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
                <button class="button" type="submit" onclick={on_submit} disabled=false>{ "Submit" }</button>
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub teams: [EntrantSpot<EntrantWithScore<Team, u64>>; 2],
    pub on_submit: Callback<[u64; 2]>,
    pub scores: [u64; 2],
}

impl PartialEq for Props {
    fn eq(&self, other: &Self) -> bool {
        self.on_submit == other.on_submit && self.scores == other.scores
    }
}

pub enum Msg {
    UpdateScore(usize, u64),
    Submit,
}
