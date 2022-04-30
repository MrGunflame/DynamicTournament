use yew::prelude::*;

use dynamic_tournament_api::tournament::Team;
use dynamic_tournament_generator::{EntrantScore, EntrantSpot};

pub struct BracketTeam;

impl Component for BracketTeam {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let text = match &ctx.props().entrant {
            EntrantSpot::Entrant(entrant) => entrant.name.clone(),
            EntrantSpot::Empty => "BYE".to_owned(),
            EntrantSpot::TBD => "TBD".to_owned(),
        };

        let (score, winner) = match &ctx.props().node {
            EntrantSpot::Entrant(node) => (node.score, node.winner),
            _ => (0, false),
        };

        let classes = if winner { "team winner" } else { "team" };

        let style = match ctx.props().color {
            Some(color) => format!("background-color: {};", color),
            None => "display: hidden;".to_owned(),
        };

        html! {
            <div class={classes}>
                <div class="team-label flex-col">
                    <div class="team-color" {style}>
                    </div>
                    <span>{ text }</span>
                </div>
                <div class="team-score">
                    { score }
                </div>
            </div>
        }
    }
}

#[derive(Clone, Debug, Properties)]
pub struct Props {
    pub entrant: EntrantSpot<Team>,
    pub node: EntrantSpot<EntrantScore<u64>>,
    pub color: Option<&'static str>,
}

impl PartialEq for Props {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
