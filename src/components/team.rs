use yew::callback::Callback;
use yew::prelude::*;

#[derive(Clone, Properties, PartialEq, Debug)]
pub struct Properties {
    pub text: String,
    pub on_score_update: Callback<u64>,
    pub score: u64,
}

pub struct Team;

impl Component for Team {
    type Message = Msg;
    type Properties = Properties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::UpdateScore => {
                ctx.props().on_score_update.emit(ctx.props().score);

                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| Msg::UpdateScore);

        let classes = match ctx.props().score {
            n if n >= 3 => "team winner",
            _ => "team",
        };

        html! {
            <div class={classes}>
                <div class="team-label">
                    {ctx.props().text.clone()}
                </div>
                <div class="team-score">
                    <button onclick={onclick}>{ctx.props().score}</button>
                </div>
            </div>
        }
    }
}

pub enum Msg {
    UpdateScore,
}
