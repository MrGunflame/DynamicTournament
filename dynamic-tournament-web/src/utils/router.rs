//! # Router
//!
//! This is an alternative router to the default `yew_router` crate. It's main feature is better
//! handling of nested routes. Nested routes only need to specify their relative behavoir, unlike
//! `yew-router` with always requires the absolute route.
//!
//! # Usage
//!
//! This module provides the [`Switch`], [`Link`] and [`Redirect`] components for use in the DOM.
//! Directly accessing the router state is also possible with [`RouterContextExt`].
//!
//! **Note that before using the router you must call [`init`] exactly once. Using any router
//! features before the function finishes execution will result in undefined behavoir.**
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

use gloo_events::EventListener;
use wasm_bindgen::JsValue;
use web_sys::MouseEvent;
use yew::html::Classes;
use yew::{html, Callback, Children, Component, Context, Html, Properties};

use crate::routes::not_found::NotFound;
use crate::statics::config;

/// The internal state of the router. Call [`state`] to get a reference to the [`State`].
///
/// # Safety
///
/// `STATE` starts as `MaybeUninit::uninit`. The consumer of this module must guarantee to call
/// [`init`] before using this router. Accessing the router state (via [`state`]) before [`init`]
/// has finished executing is undefined behavoir.
static mut STATE: MaybeUninit<State> = MaybeUninit::uninit();

/// Initializes the internal router state.
///
/// Note that calling `init` multiple times will cause the old state to leak. There is no way to
/// drop the previous router state.
///
/// # Safety
///
/// This function must not be called again while it still executes.
#[inline]
pub unsafe fn init() {
    STATE.write(State::new());
}

#[inline]
fn state() -> &'static State {
    // SAFETY: The consumer guarantees that `init` has finished executing
    // before state is called.
    unsafe { STATE.assume_init_ref() }
}

/// The active state of the router.
#[derive(Debug)]
pub struct State {
    history: web_sys::History,
    /// The current absolute path of the url. This does not include the root prefix.
    path: RefCell<PathBuf>,
    /// A list of active switches waiting for an url change.
    switches: RefCell<SwitchList>,
    /// Listener for the popstate event. This event will be fired when the browser changes
    /// the url path directly (e.g. using Forward/Back actions).
    _listener: EventListener,
}

fn strip_root(path: &mut String) {
    let root = config().root();

    let root = match root.strip_suffix('/') {
        Some(root) => root,
        None => root,
    };

    if let Some(s) = path.strip_prefix(root) {
        log::debug!("Stripping root prefix: {:?}", root);
        *path = s.to_owned();
    }
}

// SAFETY: Running in a single-threaded context. There is no way for multiple
// threads to access this.
unsafe impl Sync for State {}

impl State {
    /// Creates a new `State`.
    pub fn new() -> Self {
        let mut path = super::document().location().unwrap().pathname().unwrap();
        strip_root(&mut path);

        let listener = EventListener::new(&super::window(), "popstate", |_| {
            let mut path = super::document().location().unwrap().pathname().unwrap();
            strip_root(&mut path);

            let state = state();
            *state.path.borrow_mut() = PathBuf::from(path);

            state.notify();
        });

        Self {
            history: super::history(),
            path: RefCell::new(PathBuf::from(path)),
            switches: RefCell::new(SwitchList::new()),
            _listener: listener,
        }
    }

    /// Pushes a new `url` onto the history stack. This will rerender any existing [`Switch`]es.
    pub fn push(&self, url: String) {
        let state = state();

        let path = PathBuf::from(url);
        *state.path.borrow_mut() = path.clone();

        let root = config().root();

        let url = format!("{}/{}", root, path);
        let path = PathBuf::from(url);

        log::debug!("State::push({:?})", path);

        state
            .history
            .push_state_with_url(&JsValue::NULL, "", Some(&path.to_string()))
            .expect("Failed to push history state");

        // TODO: Don't wake when the url doesn't change.
        self.notify();
    }

    /// Update the current url, pushing a new one if it changes.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut PathBuf),
    {
        let mut path = super::document().location().unwrap().pathname().unwrap();
        strip_root(&mut path);

        let mut path = PathBuf::from(path);

        f(&mut path);

        self.push(path.to_string());
    }

    /// Notify all switches that the path changed.
    pub fn notify(&self) {
        SwitchList::wake();
    }
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct LinkProps {
    pub children: Children,
    #[prop_or_default]
    pub classes: Classes,
    pub to: String,
}

/// A `Link` component.
///
/// This is a wrapper around an `<a>` tag using the router instead of causing the site to reload.
#[derive(Debug)]
pub struct Link {
    _priv: (),
}

impl Component for Link {
    type Message = ();
    type Properties = LinkProps;

    #[inline]
    fn create(_ctx: &Context<Self>) -> Self {
        Self { _priv: () }
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: ()) -> bool {
        state().push(ctx.props().to.clone());
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|event: MouseEvent| {
            event.prevent_default();
        });

        let classes = ctx.props().classes.clone();
        let href = {
            let root = match config().root().strip_suffix('/') {
                Some(root) => root,
                None => config().root(),
            };

            format!("{}{}", root, ctx.props().to.clone())
        };

        html! {
            <a class={classes} {href} {onclick}>
                { for ctx.props().children.iter() }
            </a>
        }
    }
}

/// A route that can be matched against.
pub trait Routable: Sized + Clone + PartialEq {
    /// Tries to parse a route from the given [`PathBuf`]. Returns `None` if there is no matching
    /// route.
    fn from_path(path: &mut PathBuf) -> Option<Self>;

    /// Converts a route into a relative path.
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
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        #[allow(clippy::vtable_address_comparisons)]
        std::rc::Rc::ptr_eq(&self.render, &other.render)
    }
}

/// A switch component matching against `R`.
///
/// A `Switch` will rerender when the url chanes.
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
    /// Creates a new render function using `R`.
    #[inline]
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
        let cb = ctx.link().callback(|_| ());
        let handle = state().switches.borrow_mut().push(cb);

        Self {
            handle,
            _marker: PhantomData,
        }
    }

    #[inline]
    fn update(&mut self, _ctx: &Context<Self>, _msg: ()) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut path = state().path.borrow_mut();

        log::debug!("Matching route: {:?}", path);

        match R::from_path(&mut path) {
            Some(route) => (ctx.props().render)(&route),
            None => html! {
                <NotFound />
            },
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // Remove the handle from the waiterlist.
        state().switches.borrow_mut().remove(self.handle);
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

    #[inline]
    fn create(_ctx: &Context<Self>) -> Self {
        Self { _priv: () }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        state().push(ctx.props().to.clone());

        html! {}
    }
}

/// An owned url path buffer. This can be used to mutate the state.
// TODO: PathBuf should track the string buffer and the segments (positions) separately
// for better performance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathBuf {
    buf: Vec<String>,
}

impl PathBuf {
    #[allow(unused)]
    #[inline]
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn take(&mut self) -> Option<String> {
        if !self.is_empty() {
            Some(self.buf.remove(0))
        } else {
            None
        }
    }

    /// Returns the number of segments in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns `true` if the buffer contains no segments.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the segment at the given `index` without checking if it in bounds.
    ///
    /// # Safety
    ///
    /// This method does not check the index. Providing the value `index >= self.len()` is
    /// undefined behavoir.
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> PathMut<'_> {
        PathMut::new(self, index)
    }

    #[allow(unused)]
    pub fn get(&self, index: usize) -> Option<PathRef<'_>> {
        let segment = self.buf.get(index)?;

        Some(PathRef::new(segment))
    }

    #[allow(unused)]
    pub fn get_mut(&mut self, index: usize) -> Option<PathMut<'_>> {
        self.buf.get(index)?;

        Some(PathMut::new(self, index))
    }

    #[inline]
    pub fn last_mut(&mut self) -> Option<PathMut<'_>> {
        if self.is_empty() {
            None
        } else {
            // SAFETY: The buffer is not empty and `self.len() - 1` is always in bounds.
            unsafe { Some(self.get_unchecked_mut(self.len() - 1)) }
        }
    }

    pub fn push<T>(&mut self, path: T)
    where
        T: AsRef<str>,
    {
        if path.as_ref().contains('/') {
            panic!("Path cannot contain '/'");
        }

        self.buf.push(path.as_ref().to_string());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathRef<'a> {
    segment: &'a String,
}

impl<'a> PathRef<'a> {
    #[inline]
    fn new(segment: &'a String) -> Self {
        Self { segment }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.segment.as_str()
    }
}

impl<'a> PartialEq<&str> for PathRef<'a> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PathMut<'a> {
    buf: &'a mut PathBuf,
    segment: usize,
}

impl<'a> PathMut<'a> {
    #[inline]
    fn new(buf: &'a mut PathBuf, segment: usize) -> Self {
        Self { buf, segment }
    }

    #[allow(unused)]
    #[inline]
    pub fn as_str(&self) -> &str {
        self.buf.buf[self.segment].as_str()
    }

    #[allow(unused)]
    pub fn push<T>(&mut self, path: T)
    where
        T: AsRef<str>,
    {
        if path.as_ref().contains('/') {
            panic!("Path cannot contain '/'");
        }

        self.buf.buf[self.segment].push_str(path.as_ref());
    }

    /// Replaces the whole segment in the buffer.
    pub fn replace<T>(&mut self, path: T)
    where
        T: AsRef<str>,
    {
        if path.as_ref().contains('/') {
            panic!("Path cannot contain '/'");
        }

        self.buf.buf[self.segment] = path.as_ref().to_string();
    }

    /// Removes the whole segment from the buffer.
    #[allow(unused)]
    #[inline]
    pub fn remove(self) {
        self.buf.buf.remove(self.segment);
    }
}

impl Display for PathBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "/{}", self.buf.join("/"))
    }
}

impl<'a> From<&'a str> for PathBuf {
    #[inline]
    fn from(path: &'a str) -> Self {
        path.to_string().into()
    }
}

impl From<String> for PathBuf {
    fn from(buf: String) -> Self {
        let buf = buf
            .split('/')
            .filter(|s| !(*s).is_empty())
            .map(|s| s.to_owned())
            .collect();

        Self { buf }
    }
}

impl<T> PartialEq<T> for PathBuf
where
    T: AsRef<str>,
{
    // Unfortunately we cannot cannot compare the path directly. We need to join the segments
    // together to create the path format used by `other`. This is not possible without cloning
    // and allocation in this case.
    // This can be resolved when `PathBuf` keeps track of the segments separately from the
    // internal string buffer.
    #[allow(clippy::cmp_owned)]
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.to_string() == other.as_ref()
    }
}

/// A list of references all currently rendered [`Switch`]es. Using [`wake`] will cause all
/// switches in the list rerender.
#[derive(Clone, Debug)]
struct SwitchList {
    list: BTreeMap<usize, Callback<()>>,
    id: usize,
}

impl SwitchList {
    #[inline]
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

    /// Removes a reference to a switch.
    pub fn remove(&mut self, handle: usize) {
        self.list.remove(&handle);
    }

    /// Wakes all switches in the list, causing them to rerender. They will be woken in the order
    /// they were registered.
    pub fn wake() {
        // Clone all active callbacks into a separate collection. Doing this is necessary if the
        // callback destroys a switch, causing `state.switches` to be borrowed mutably.
        let switches = state().switches.borrow();
        let list: Vec<_> = switches.list.iter().map(|(_, cb)| cb.clone()).collect();
        drop(switches);

        log::debug!("Waking {} waiting switches", list.len());

        for cb in list {
            cb.emit(());
        }
    }
}

/// An extension trait for [`yew::Context`] that provides direct access to the routers [`State`].
pub trait RouterContextExt {
    /// Returns a reference to the routers [`State`].
    fn router(&self) -> &'static State;
}

impl<C> RouterContextExt for yew::Context<C>
where
    C: yew::Component,
{
    #[inline]
    fn router(&self) -> &'static State {
        state()
    }
}

#[cfg(test)]
mod tests {
    use super::PathBuf;

    #[test]
    fn test_path_buf_push() {
        let mut path = PathBuf::new();
        path.push("index");
        assert_eq!(path, "/index");

        path.push("test");
        assert_eq!(path, "/index/test");

        assert_eq!(path.get(0).unwrap(), "index");
        assert_eq!(path.get(1).unwrap(), "test");
        assert_eq!(path.get(2), None);
    }

    #[test]
    fn test_path_buf_mut() {
        let mut path = PathBuf::from("/a/b/c");
        assert_eq!(path, "/a/b/c");

        let mut seg = path.get_mut(0).unwrap();
        seg.push("test");
        assert_eq!(path, "/atest/b/c");

        let mut seg = path.get_mut(1).unwrap();
        seg.replace("path");
        assert_eq!(path, "/atest/path/c");

        let seg = path.get_mut(1).unwrap();
        seg.remove();
        assert_eq!(path, "/atest/c");
    }

    #[test]
    fn test_path_buf_from_string() {
        let path = PathBuf::from("/a/b/c");
        assert_eq!(path, "/a/b/c");
    }
}
