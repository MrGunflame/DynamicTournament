use std::borrow::Cow;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use std::time::Duration;

use asyncsync::local::{Notified, Notify};
use chrono::Utc;
use dynamic_tournament_api::{auth::Token, Client as InnerClient, Result};
use futures::{select, FutureExt};
use gloo_timers::future::sleep;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Debug)]
pub struct Client {
    inner: InnerClient,
    // Waker to wake the background sleep
    waker: Rc<Notify>,
    // TODO: Use a broadcast channel to merge these
    on_login: Rc<Notify>,
    on_logout: Rc<Notify>,
}

impl Client {
    /// Creates a new `Client` with the given `base_url`.
    #[inline]
    pub fn new<T>(base_url: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let this = Self {
            inner: InnerClient::new(base_url),
            waker: Rc::new(Notify::new()),
            on_login: Rc::new(Notify::new()),
            on_logout: Rc::new(Notify::new()),
        };

        if let Some(token) = this.authorization().refresh_token() {
            if token_lifetime(token) >= 30 {
                this.spawn_refresh();
            }
        }

        this
    }

    /// Tries to log in using the provided `username` and `password`.
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        self.inner.v3().auth().login(username, password).await?;

        self.on_login.notify_all();
        self.spawn_refresh();

        Ok(())
    }

    /// Logs the `Client` out and removes any authentication information.
    pub fn logout(&self) {
        self.inner.logout();

        // Stop all sleep futures.
        self.waker.notify_all();
    }

    /// Returns an [`Action`] when the state of the client changes.
    pub fn changed(&self) -> Changed<'_> {
        Changed {
            on_login: self.on_login.notified(),
            on_logout: self.on_logout.notified(),
        }
    }

    /// Returns the current [`State`] of the `Client`.
    pub fn state(&self) -> State {
        // Skips validating the actual token currently.
        match self.inner.authorization().refresh_token() {
            Some(_) => State::LoggedIn,
            None => State::LoggedOut,
        }
    }

    /// Spawns a new refresh task on the current task.
    ///
    /// Note: The task runs util it is destroyed using `self.waker`.
    fn spawn_refresh(&self) {
        let client = self.clone();

        spawn_local(async move {
            loop {
                match client.inner.authorization().auth_token() {
                    Some(token) => {
                        let now = Utc::now().timestamp() as u64;
                        let lifetime = token.claims().exp.saturating_sub(now + 30);

                        log::debug!("Auth token is valid for {}s", lifetime);

                        select! {
                            // Sleep until the auth token expires.
                            _ = sleep(Duration::new(lifetime, 0)).fuse() => {
                                log::debug!("Refreshing auth tokens");
                                client.refresh().await;
                            }
                            _ = client.waker.notified() => {
                                log::debug!("Interrupt sleep future");
                                break;
                            }
                        }
                    }
                    // No valid auth token exists. If we still have a valid refresh token
                    // we can acquire a new auth token.
                    None => match client.inner.authorization().refresh_token() {
                        // Check if the token is still valid.
                        Some(token) if token_lifetime(token) >= 30 => {
                            client.refresh().await;
                        }
                        // No refresh token: We cannot acquire a new token, exit.
                        _ => {
                            // Logout of the inner client. This will delete existing
                            // the token from storage.
                            client.inner.logout();
                        }
                    },
                }
            }

            client.on_logout.notify_all();
            log::debug!("Refresh token expired");
        });
    }

    /// Try to refresh the authentication tokens while the refresh token is still valid.
    ///
    /// This method will only return once the tokens have been refreshed successfully or the
    /// refresh token expired. In this case the `Client` is logged out.
    async fn refresh(&self) {
        // Get the remaining lifetime of the refresh token.
        let mut lifetime = match self.authorization().refresh_token() {
            Some(token) => token.claims().exp - Utc::now().timestamp() as u64,
            None => return,
        };

        if lifetime == 0 {
            return;
        }

        loop {
            match self.inner.v3().auth().refresh().await {
                Ok(()) => return,
                Err(err) => log::error!("Failed to refresh: {:?}", err),
            }

            // Check before sleeping.
            lifetime = lifetime.saturating_sub(30);
            if lifetime == 0 {
                self.logout();
                return;
            }

            sleep(Duration::new(30, 0)).await;
        }
    }

    pub fn is_authenticated(&self) -> bool {
        if let Some(token) = self.authorization().refresh_token() {
            if token_lifetime(token) >= 30 {
                return true;
            }
        }

        false
    }
}

impl Deref for Client {
    type Target = InnerClient;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

/// An change action from a [`Client`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Login,
    Logout,
}

pub struct Changed<'a> {
    on_login: Notified<'a>,
    on_logout: Notified<'a>,
}

impl<'a> Changed<'a> {
    fn poll_on_login(self: &mut Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Action> {
        let on_login = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.on_login) };

        on_login.poll(cx).map(|_| Action::Login)
    }

    fn poll_on_logout(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Action> {
        let on_logout = unsafe { self.map_unchecked_mut(|this| &mut this.on_logout) };

        on_logout.poll(cx).map(|_| Action::Logout)
    }
}

impl<'a> Future for Changed<'a> {
    type Output = Action;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.poll_on_login(cx) {
            Poll::Ready(a) => Poll::Ready(a),
            Poll::Pending => match self.poll_on_logout(cx) {
                Poll::Ready(a) => Poll::Ready(a),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}

/// Returns the remaining lifetime of the `token`.
fn token_lifetime(token: &Token) -> u64 {
    token.claims().exp - Utc::now().timestamp() as u64
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum State {
    LoggedIn,
    LoggedOut,
}
