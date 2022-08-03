use std::borrow::Cow;
use std::ops::Deref;
use std::time::Duration;
use std::rc::Rc;

use chrono::Utc;
use dynamic_tournament_api::{Client as InnerClient, Result};
use gloo_timers::future::sleep;
use wasm_bindgen_futures::spawn_local;
use asyncsync::local::Notify;
use futures::{select, FutureExt};

#[derive(Clone, Debug)]
pub struct Client {
    inner: InnerClient,
    // Waker to wake the background sleep
    waker: Rc<Notify>,
}

impl Client {
    /// Creates a new `Client` with the given `base_url`.
    #[inline]
    pub fn new<T>(base_url: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            inner: InnerClient::new(base_url),
            waker: Rc::new(Notify::new()),
        }
    }

    /// Tries to log in using the provided `username` and `password`.
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        self.inner.v3().auth().login(username, password).await?;

        let client = self.clone();
        spawn_local(async move {
            while let Some(token) = client.inner.authorization().auth_token() {
                let now = Utc::now().timestamp() as u64;
                let seconds = token.claims().exp.saturating_sub(now + 30);

                log::debug!("Auth token is valid for {}s", seconds);

                select! {
                    _ = sleep(Duration::new(seconds, 0)).fuse() => {
                        log::debug!("Refreshing auth tokens");
                        client.refresh().await;
                    }
                    _ = client.waker.notified() => {
                        log::debug!("Interrupt sleep future");
                        break;
                    }
                }
            }

            log::debug!("Refresh token expired");
        });

        Ok(())
    }

    /// Logs the `Client` out and removes any authentication information.
    pub fn logout(&self) {
        self.inner.logout();

        // Stop all sleep futures.
        self.waker.notify_all();
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
