use yew::prelude::*;

use crate::api::tournament::Team;
use crate::bracket_generator::{EntrantSpot, EntrantWithScore};

pub struct BracketTeam;

impl Component for BracketTeam {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (text, score, winner) = match &ctx.props().entrant {
            EntrantSpot::Entrant(entrant) => {
                (entrant.entrant.name.clone(), entrant.score, entrant.winner)
            }
            EntrantSpot::Empty => ("BYE".to_owned(), 0, false),
            EntrantSpot::TBD => ("TBD".to_owned(), 0, false),
        };

        let classes = if winner { "team winner" } else { "team" };

        html! {
            <div class={classes}>
                <div class="team-label">
                    { text }
                </div>
                <div class="team-score">
                    { score }
                </div>
            </div>
        }
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub entrant: EntrantSpot<EntrantWithScore<Team, u64>>,
}
