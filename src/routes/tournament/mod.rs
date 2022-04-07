mod teamdetails;
mod teams;

use teamdetails::TeamDetails;
use teams::Teams;

use reqwasm::http::Request;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::api::tournament as api;
use crate::api::v1::tournament as api2;
use crate::components::bracket::Bracket;
use crate::components::config_provider::Config;
use crate::{render_data, Data, DataResult};

pub struct Tournament {
    data: Data<(api::Tournament, Option<api2::Bracket>)>,
}

impl Component for Tournament {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let link = ctx.link();
        let (config, _) = ctx.link().context::<Config>(Callback::noop()).unwrap();

        let id = ctx.props().id;
        link.send_future(async move {
            async fn fetch_data(
                config: Config,
                id: u64,
            ) -> DataResult<(api::Tournament, Option<api2::Bracket>)> {
                let data = Request::get(&format!("{}/api/v1/tournament/{}", config.api_url, id))
                    .send()
                    .await?
                    .json()
                    .await?;

                let bracket = match api2::Bracket::get(id, config).await {
                    Ok(bracket) => Some(bracket),
                    Err(_) => None,
                };

                Ok((data, bracket))
            }

            let data = Some(fetch_data(config, id).await);

            Msg::Update(data)
        });

        Self { data: None }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Update(data) => {
                self.data = data;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        render_data(&self.data, |(data, bracket)| {
            let rc = std::rc::Rc::new(data.clone());

            let bracket = bracket.clone();
            let switch = move |route: &Route| -> Html {
                let rc = rc.clone();
                let bracket = match bracket.clone() {
                    Some(bracket) => Some(std::rc::Rc::new(bracket)),
                    None => None,
                };

                match route {
                    Route::Index { id } => html! {
                        <span>{ format!("Tournament id {}", id) }</span>
                    },
                    Route::Bracket { id } => html! {
                        <Bracket tournament={rc} bracket={bracket} />
                    },
                    Route::Teams { id } => html! {
                        <Teams teams={rc} />
                    },
                    Route::TeamDetails { id, team_id } => html! {
                        <TeamDetails teams={rc} index={*team_id} />
                    },
                }
            };

            html! {
                <>
                    <div class="navbar">
                        <ul>
                            <li><Link<Route> to={Route::Bracket{ id: ctx.props().id }}>{ "Bracket" }</Link<Route>></li>
                            <li><Link<Route> to={Route::Teams{ id: ctx.props().id }}>{ "Teams" }</Link<Route>></li>
                        </ul>
                    </div>
                    <Switch<Route> render={Switch::render(switch)} />
                </>
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub id: u64,
}

pub enum Msg {
    Update(Data<(api::Tournament, Option<api2::Bracket>)>),
}

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/tournament/:id")]
    Index { id: u64 },
    #[at("/tournament/:id/bracket")]
    Bracket { id: u64 },
    #[at("/tournament/:id/teams")]
    Teams { id: u64 },
    #[at("/tournament/:id/teams/:team_id")]
    TeamDetails { id: u64, team_id: u32 },
}
