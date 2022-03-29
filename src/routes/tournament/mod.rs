mod bracket;
mod teamdetails;
mod teams;

use bracket::Bracket;
use teamdetails::TeamDetails;
use teams::Teams;

use reqwasm::http::Request;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::{render_data, Data, DataResult};

pub struct Tournament {
    // data: Option<crate::MatchmakerInput>,
    data: Data<crate::MatchmakerInput>,
}

impl Component for Tournament {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();

        link.send_future(async move {
            async fn fetch_data() -> DataResult<crate::MatchmakerInput> {
                let data = Request::get("http://localhost:8000/data.json")
                    .send()
                    .await?
                    .json()
                    .await?;

                Ok(data)
            }

            let data = Some(fetch_data().await);

            Msg::Update(data)
        });

        Self { data: None }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(data) => {
                self.data = data;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        render_data(&self.data, |data| {
            let rc = std::rc::Rc::new(data.clone());

            let switch = move |route: &Route| -> Html {
                let rc = rc.clone();

                match route {
                    Route::Index => html! {
                        <span>{"this is index"}</span>
                    },
                    Route::Bracket => html! {
                        <Bracket tournament={rc} />
                    },
                    Route::Teams => html! {
                        <Teams teams={rc} />
                    },
                    Route::TeamDetails { id } => html! {
                        <TeamDetails teams={rc} index={*id} />
                    },
                }
            };

            html! {
                <BrowserRouter>
                    <div class="navbar">
                        <ul>
                            <li><Link<Route> to={Route::Index}>{ "Home" }</Link<Route>></li>
                            <li><Link<Route> to={Route::Bracket}>{ "Bracket" }</Link<Route>></li>
                            <li><Link<Route> to={Route::Teams}>{ "Teams" }</Link<Route>></li>
                        </ul>
                    </div>
                    <Switch<Route> render={Switch::render(switch)} />
                </BrowserRouter>
            }
        })
    }
}

pub enum Msg {
    Update(Data<crate::MatchmakerInput>),
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Index,
    #[at("/bracket")]
    Bracket,
    #[at("/teams")]
    Teams,
    #[at("/teams/:id")]
    TeamDetails { id: u32 },
}
