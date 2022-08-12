use std::marker::PhantomData;

use wasm_bindgen::JsValue;
use web_sys::MouseEvent;
use yew::context::ContextProvider;
use yew::{html, Callback, Children, Component, Context, Html, Properties};

use super::{history, Rc};

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

#[derive(Debug)]
pub struct Router {
    stack: Vec<String>,
}

impl Component for Router {
    type Message = String;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self { stack: Vec::new() }
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: String) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let children = &ctx.props().children;

        let callback = ctx.link().callback(|url| url);
        let history = History::new(callback);

        html! {
            <>
                <ContextProvider<History> context={history}>
                    { for children.iter() }
                </ContextProvider<History>>
            </>
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct History {
    history: Rc<web_sys::History>,
    callback: Callback<String>,
}

impl History {
    pub fn new(cb: Callback<String>) -> Self {
        Self {
            history: Rc::new(history()),
            callback: cb,
        }
    }

    pub fn push(&self, url: String) {
        self.history
            .push_state_with_url(&JsValue::NULL, "", Some(&url))
            .expect("Failed to push history");

        self.callback.emit(url);
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LinkProps<R>
where
    R: Routable + PartialEq,
{
    pub children: Children,
    pub to: R,
}

#[derive(Debug)]
pub struct Link<R>
where
    R: Routable + PartialEq + 'static,
{
    _marker: PhantomData<R>,
}

impl<R> Component for Link<R>
where
    R: Routable + PartialEq + 'static,
{
    type Message = ();
    type Properties = LinkProps<R>;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: ()) -> bool {
        let (history, _) = ctx.link().context::<History>(Callback::noop()).unwrap();

        history.push(ctx.props().to.to_path());
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|event: MouseEvent| {
            event.prevent_default();
        });

        html! {
            <a href="/" {onclick}>
                { for ctx.props().children.iter() }
            </a>
        }
    }
}

pub trait Routable: Sized + Clone + PartialEq {
    fn from_path(path: &mut Path) -> Option<Self>;

    fn to_path(&self) -> String;

    fn not_found() -> Option<Self> {
        None
    }
}

#[derive(Properties)]
pub struct SwitchProps<R>
where
    R: PartialEq,
{
    pub render: std::rc::Rc<dyn Fn(&R) -> Html>,
}

impl<R> PartialEq for SwitchProps<R>
where
    R: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::vtable_address_comparisons)]
        std::rc::Rc::ptr_eq(&self.render, &other.render)
    }
}

pub struct Switch<R>
where
    R: Routable,
{
    _marker: PhantomData<R>,
}

impl<R> Switch<R>
where
    R: Routable,
{
    pub fn render<F>(f: F) -> std::rc::Rc<dyn Fn(&R) -> Html>
    where
        F: Fn(&R) -> Html + 'static,
    {
        std::rc::Rc::new(f)
    }
}

impl<R> Component for Switch<R>
where
    R: Routable + 'static,
{
    type Message = ();
    type Properties = SwitchProps<R>;

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match R::from_path("") {
            Some(route) => (ctx.props().render)(&route),
            None => html! { "Oh no" },
        }
    }
}

pub struct Path {
    path: String,
    pos: usize,
}

impl Path {
    fn new(path: String) -> Self {
        Self { path, pos:0  }
    }

    pub fn take(&mut self) -> Option<&str> {
        let mut end = self.pos;
        loop {
            match self.path.chars().next() {
                Some(c) if c == '/' => break,
                Some(_) => end +=1,
                None => break,
            }
        }

        let path = &self.path[self.pos..end];
        self.pos = end;

        Some(path)
    }
}
