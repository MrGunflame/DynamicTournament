use std::fmt::Display;
use std::marker::PhantomData;
use std::str::FromStr;

use yew::{html, Callback, Classes, Component, Context, Html, Properties};

use crate::components::Input;

#[derive(Clone, Debug, Properties)]
pub struct Props<T>
where
    T: FromStr + 'static,
    T::Err: Display,
{
    pub value: Option<String>,
    pub onchange: Callback<T>,
    #[prop_or("text")]
    pub kind: &'static str,
    #[prop_or_default]
    pub classes: Classes,
}

impl<T> PartialEq for Props<T>
where
    T: FromStr + 'static,
    T::Err: Display,
{
    fn eq(&self, other: &Props<T>) -> bool {
        self.onchange == other.onchange
    }
}

/// An `<input />` element that expects a valid `T` to be input.
/// If the input is invalid, the `T::Err` value is displayed, otherwise the parsed `T` is returned.
pub struct ParseInput<T>
where
    T: FromStr + 'static,
    T::Err: Display,
{
    value: String,
    error: Option<String>,
    _marker: PhantomData<T>,
}

impl<T> Component for ParseInput<T>
where
    T: FromStr + 'static,
    T::Err: Display,
{
    type Message = String;
    type Properties = Props<T>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            value: ctx.props().value.clone().unwrap_or_default(),
            error: None,
            _marker: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: String) -> bool {
        self.value = msg;

        match T::from_str(&self.value) {
            Ok(val) => {
                self.error = None;
                ctx.props().onchange.emit(val);
            }
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let error = self
            .error
            .as_ref()
            .map(|error| {
                html! {
                    <span>{ error }</span>
                }
            })
            .unwrap_or_else(|| html! {});

        let onchange = ctx.link().callback(|val: String| val);
        let value = self.value.clone();
        let classes = ctx.props().classes.clone();

        html! {
            <div>
                <Input kind="text" {value} {onchange} {classes} />
                { error }
            </div>
        }
    }
}
