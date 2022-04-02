use yew::prelude::*;

#[derive(Clone, Properties, PartialEq, Debug)]
pub struct Properties {
    pub text: String,
    pub score: u64,
    pub is_winner: bool,
}

pub struct Team;

impl Component for Team {
    type Message = ();
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let classes = if ctx.props().is_winner {
            "team winner"
        } else {
            "team"
        };

        html! {
            <div class={classes}>
                <div class="team-label">
                    {ctx.props().text.clone()}
                </div>
                <div class="team-score">
                    {ctx.props().score}
                </div>
            </div>
        }
    }
}
