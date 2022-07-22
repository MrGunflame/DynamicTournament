use dynamic_tournament_api::v3::systems::SystemOverview;
use yew::{html, Component, Context, Html};

use crate::components::providers::{ClientProvider, Provider};
use crate::utils::FetchData;

#[derive(Debug)]
pub struct Systems {
    systems: FetchData<Vec<SystemOverview>>,
}

impl Component for Systems {
    type Message = Message;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let client = ClientProvider::get(ctx);
        ctx.link().send_future(async move {
            let msg = match client.v3().systems().list().await {
                Ok(systems) => FetchData::from(systems),
                Err(err) => FetchData::from_err(err),
            };

            Message::UpdateSystems(msg)
        });

        Self {
            systems: FetchData::new(),
        }
    }

    #[inline]
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Message::UpdateSystems(systems) => self.systems = systems,
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        self.systems.render(|systems| {
            let body: Html = systems.iter().map(render_system).collect();

            html! {
                <div>
                    <table>
                        <thead>
                            <tr>
                                <th>{ "ID" }</th>
                                <th>{ "Name" }</th>
                            </tr>
                        </thead>
                        <tbody>
                            { body }
                        </tbody>
                    </table>
                </div>
            }
        })
    }
}

#[derive(Debug)]
pub enum Message {
    UpdateSystems(FetchData<Vec<SystemOverview>>),
}

#[inline]
fn render_system(system: &SystemOverview) -> Html {
    html! {
        <tr>
            <td>{ system.id }</td>
            <td>{ system.name.clone() }</td>
        </tr>
    }
}
