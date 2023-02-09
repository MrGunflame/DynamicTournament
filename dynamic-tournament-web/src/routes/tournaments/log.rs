use dynamic_tournament_api::auth::Flags;
use dynamic_tournament_api::v3::tournaments::log::{LogEvent, LogEventBody};
use dynamic_tournament_api::v3::tournaments::Tournament;
use yew::{html, Component, Html, Properties};

use crate::components::protected::ForbiddenAction;
use crate::components::providers::{ClientProvider, Provider};
use crate::components::{Protected, Time};
use crate::utils::{FetchData, Rc};

#[derive(Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub tournament: Rc<Tournament>,
}

pub struct Log;

impl Component for Log {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <Protected flags={Flags::ADMIN} action={ForbiddenAction::NotFound}>
                <LogInner tournament={ctx.props().tournament.clone()} />
            </Protected>
        }
    }
}

/// Hide the actual log component behind a separate component.
/// This makes it easier to only make requests after the protected children were rendered.
#[derive(Debug)]
struct LogInner {
    events: FetchData<Vec<LogEvent>>,
}

impl Component for LogInner {
    type Message = FetchData<Vec<LogEvent>>;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);

        let id = ctx.props().tournament.id;
        ctx.link().send_future(async move {
            match client.v3().tournaments().log(id).list().await {
                Ok(val) => FetchData::from(val),
                Err(err) => FetchData::from_err(err),
            }
        });

        Self {
            events: FetchData::new(),
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        self.events = msg;
        true
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        self.events.render(|events| {
            let events = events
                .iter()
                .map(|event| {
                    let body = match event.body {
                        LogEventBody::UpdateMatch {
                            bracket_id,
                            index,
                            nodes,
                        } => {
                            format!(
                                "Updated match {} to {}-{} (Bracket {})",
                                index, nodes[0].score, nodes[1].score, bracket_id
                            )
                        }
                        LogEventBody::ResetMatch { bracket_id, index } => {
                            format!("Reset match {} (Bracket {})", index, bracket_id)
                        }
                    };

                    html! {
                        <div class="dt-evlog">
                            <div class="dt-evlog-author">{ event.author }</div>
                            <div>{ body }</div>
                            <div class="dt-evlog-date"><Time datetime={event.date} /></div>
                        </div>
                    }
                })
                .collect::<Html>();

            html! {
                <div>
                    { events }
                </div>
            }
        })
    }
}
