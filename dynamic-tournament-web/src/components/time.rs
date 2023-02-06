use chrono::{DateTime, Utc};
use yew::{html, Component, Properties};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Properties)]
pub struct Props {
    pub datetime: DateTime<Utc>,
}

/// Time formatter
pub struct Time {
    _priv: (),
}

impl Component for Time {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self { _priv: () }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let now = Utc::now();

        let duration = now - ctx.props().datetime;

        if duration.num_days() >= 1 {
            html! {
                format!("{} days ago", duration.num_days())
            }
        } else if duration.num_hours() >= 1 {
            html! {
                format!("{} hours ago", duration.num_hours())
            }
        } else if duration.num_minutes() >= 1 {
            html! {
                format!("{} minutes ago",duration.num_minutes())
            }
        } else {
            html! {
                format!("{} second ago", duration.num_seconds())
            }
        }
    }
}
