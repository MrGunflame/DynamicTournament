use yew::callback::Callback;
use yew::prelude::*;

use super::team::Team;

use std::cell::RefCell;
use std::rc::Rc;

pub struct Match {
    scores: Rc<RefCell<[u64; 2]>>,
}

impl Component for Match {
    type Message = Msg;
    type Properties = MatchProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            scores: Rc::new(RefCell::new([0; 2])),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update => {
                let scores = { (*self.scores).borrow().clone() };

                if let Some(winner) = scores
                    .into_iter()
                    .enumerate()
                    .filter(|(_, score)| *score >= 3)
                    .map(|(i, _)| i)
                    .next()
                {
                    ctx.props().on_winner_update.emit(winner as usize);
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let teams: Html = ctx.props().teams
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, team)| {
                let cell = self.scores.clone();
                let cb = link.callback(move |score: u64| {
                    (*cell).borrow_mut()[i] = score + 1;
                    Msg::Update
                });
                // let cb = Callback::from(move |score: u64| {
                //     (*cell).borrow_mut()[i] = score + 1;
                // });

                let score = (*self.scores).borrow()[i];

                match team {
                    MatchMember::Entrant(team) => {
                        html! {<Team text={team.name.clone()} on_score_update={cb.clone()} score={score} />}
                    }
                    MatchMember::Placeholder(s) => {
                        let clos = Callback::from(|_:u64| {});

                        html! {
                            <Team text={s} on_score_update={clos} score={0} />
                        }
                    },
                }
            })
            .collect();

        html! {
            <div class="match">
                {teams}
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct MatchProperties {
    pub teams: [MatchMember; 2],
    // Returns the index of the winning team (either 0 or 1).
    pub on_winner_update: Callback<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MatchMember {
    Entrant(crate::Team),
    Placeholder(String),
}

pub enum Msg {
    Update,
}
