use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;

use wasm_bindgen::JsValue;
use web_sys::MouseEvent;
use yew::context::ContextProvider;
use yew::html::Classes;
use yew::{html, Callback, Children, Component, Context, Html, Properties};

use super::{history, Rc};
use crate::statics::config;

#[derive(Debug, PartialEq, Properties)]
pub struct Props {
    pub children: Children,
}

#[derive(Debug)]
pub struct Router {
    history: History,
}

impl Component for Router {
    type Message = String;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let callback = ctx.link().callback(|url| url);
        let history = History::new(callback);

        Self { history }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: String) -> bool {
        let mut state = self.history.state.borrow_mut();
        state.path = Path::new(msg);

        let mut switches = self.history.switches.borrow_mut();
        switches.wake();

        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let history = self.history.clone();

        html! {
            <>
                <ContextProvider<History> context={history}>
                    { for ctx.props().children.iter() }
                </ContextProvider<History>>
            </>
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    path: Path,
}

#[derive(Clone, Debug, PartialEq)]
pub struct History {
    history: Rc<web_sys::History>,
    callback: Callback<String>,
    state: Rc<RefCell<State>>,
    // Vec of switches in registered order
    switches: Rc<RefCell<SwitchList>>,
}

impl History {
    pub fn new(cb: Callback<String>) -> Self {
        let path = super::document()
            .location()
            .expect("no document.location")
            .pathname()
            .expect("failed to fetch location pathname");

        Self {
            history: Rc::new(history()),
            callback: cb,
            state: Rc::new(RefCell::new(State {
                path: Path::new(path),
            })),
            switches: Rc::new(RefCell::new(SwitchList::new())),
        }
    }

    pub fn push(&self, url: String) {
        let root = config().root();

        let mut seg = url.as_str();
        if url.starts_with('/') {
            seg = url.strip_prefix('/').unwrap();
        }

        let mut url = if root.ends_with('/') {
            format!("{}{}", root, seg)
        } else {
            format!("{}/{}", root, seg)
        };

        // history.pushState doesn't allow passing an empty string as the url.
        // Pass a "/" instead.
        if url.is_empty() {
            url.push('/');
        }

        log::debug!("History::push {:?}", url);

        self.history
            .push_state_with_url(&JsValue::NULL, "", Some(&url))
            .expect("Failed to push history");

        self.callback.emit(url);
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut PathBuf),
    {
        let path = super::document().location().unwrap().pathname().unwrap();
        let mut path = PathBuf::new(path);

        f(&mut path);

        self.push(path.to_string());
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LinkProps {
    pub children: Children,
    #[prop_or_default]
    pub classes: Classes,
    pub to: String,
}

#[derive(Debug)]
pub struct Link {
    _priv: (),
}

impl Component for Link {
    type Message = ();
    type Properties = LinkProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { _priv: () }
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: ()) -> bool {
        let (history, _) = ctx.link().context::<History>(Callback::noop()).unwrap();

        history.push(ctx.props().to.clone());
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|event: MouseEvent| {
            event.prevent_default();
        });

        let classes = ctx.props().classes.clone();
        let href = ctx.props().to.clone();

        html! {
            <a class={classes} {href} {onclick}>
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
    handle: usize,
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
        let (history, _) = ctx
            .link()
            .context::<History>(Callback::noop())
            .expect("no router installed");

        let mut switches = history.switches.borrow_mut();

        let cb = ctx.link().callback(|_| ());
        let handle = switches.push(cb);

        Self {
            handle,
            _marker: PhantomData,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: ()) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (history, _) = ctx
            .link()
            .context::<History>(Callback::noop())
            .expect("no router installed");
        let mut state = history.state.borrow_mut();

        log::debug!("Matching route: {:?}", state);

        match R::from_path(&mut state.path) {
            Some(route) => (ctx.props().render)(&route),
            None => html! { "Oh no" },
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        let (history, _) = ctx
            .link()
            .context::<History>(Callback::noop())
            .expect("no router installed");

        let mut switches = history.switches.borrow_mut();
        switches.remove(self.handle);
    }
}

#[derive(Debug, PartialEq, Eq, Properties)]
pub struct RedirectProps {
    pub to: String,
}

#[derive(Debug)]
pub struct Redirect {
    _priv: (),
}

impl Component for Redirect {
    type Message = ();
    type Properties = RedirectProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { _priv: () }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (history, _) = ctx.link().context::<History>(Callback::noop()).unwrap();
        history.push(ctx.props().to.clone());

        html! {}
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathBuf {
    segments: Vec<String>,
}

impl PathBuf {
    fn new(path: String) -> Self {
        let parts = path
            .split('/')
            .filter(|s| !(*s).is_empty())
            .map(|s| s.to_string())
            .collect();

        Self { segments: parts }
    }

    pub fn last_mut(&mut self) -> Option<&mut String> {
        self.segments.last_mut()
    }

    pub fn push<T>(&mut self, segment: T)
    where
        T: ToString,
    {
        self.segments.push(segment.to_string());
    }
}

impl Display for PathBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.segments.join("/"))
    }
}

#[derive(Clone)]
pub struct Path {
    parts: Vec<String>,
    pos: usize,
}

impl Path {
    fn new(path: String) -> Self {
        let parts = path
            .split('/')
            .filter(|s| !(*s).is_empty())
            .map(|s| s.to_string())
            .collect();

        Self { parts, pos: 0 }
    }

    pub fn take(&mut self) -> Option<&str> {
        let path = self.parts.get(self.pos)?;
        self.pos += 1;

        log::debug!("Taking part {}: {:?}", self.pos - 1, path);

        Some(path)
    }
}

impl Debug for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.parts.join("/"))
    }
}

#[derive(Clone, Debug)]
struct SwitchList {
    list: BTreeMap<usize, Callback<()>>,
    id: usize,
}

impl SwitchList {
    fn new() -> Self {
        Self {
            list: BTreeMap::new(),
            id: 0,
        }
    }

    /// Pushes a new switch to the list and returns a handle to it.
    pub fn push(&mut self, cb: Callback<()>) -> usize {
        let id = self.id;
        self.id += 1;
        self.list.insert(id, cb);

        id
    }

    pub fn remove(&mut self, handle: usize) {
        self.list.remove(&handle);
    }

    pub fn wake(&mut self) {
        log::debug!("Waking {} waiting switches", self.list.len());

        for cb in self.list.values() {
            cb.emit(());
        }
    }
}

pub trait RouterContextExt {
    fn history(&self) -> History;
}

impl<C> RouterContextExt for yew::Context<C>
where
    C: yew::Component,
{
    fn history(&self) -> History {
        let (h, _) = self
            .link()
            .context::<History>(yew::Callback::noop())
            .expect("no router");
        h
    }
}

#[cfg(test)]
mod tests {
    use super::Path;

    #[test]
    fn test_path_take() {
        let mut path = Path::new(String::from(""));
        assert_eq!(path.take(), None);

        let mut path = Path::new(String::from("/"));
        assert_eq!(path.take(), None);

        let mut path = Path::new(String::from("/a/b"));
        assert_eq!(path.take(), Some("a"));
        assert_eq!(path.take(), Some("b"));
        assert_eq!(path.take(), None);
    }
}
