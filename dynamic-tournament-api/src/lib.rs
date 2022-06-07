pub mod auth;
pub mod http;
pub mod tournament;
pub mod v3;
pub mod websocket;

use crate::auth::AuthClient;
use crate::http::{Request, RequestBuilder, Response};
use crate::tournament::TournamentClient;

use ::http::StatusCode;
use auth::TokenPair;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::sync::{Arc, RwLock};

#[cfg(feature = "local-storage")]
use gloo_storage::{LocalStorage, Storage};

#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<RwLock<ClientInner>>,
    client: http::Client,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        let inner = ClientInner {
            base_url,
            authorization: Authorization::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
            client: http::Client::new(),
        }
    }

    pub fn v3(&self) -> v3::Client {
        v3::Client::new(self)
    }

    pub fn tournaments(&self) -> TournamentClient<'_> {
        TournamentClient::new(self)
    }

    pub fn auth(&self) -> AuthClient {
        AuthClient::new(self)
    }

    pub(crate) fn request(&self) -> RequestBuilder {
        let inner = self.inner.read().unwrap();

        RequestBuilder::new(inner.base_url.clone(), &inner.authorization)
    }

    pub fn is_authenticated(&self) -> bool {
        let inner = self.inner.read().unwrap();
        inner.authorization.tokens.is_some()
    }

    pub fn authorization(&self) -> Authorization {
        let inner = self.inner.read().unwrap();

        inner.authorization.clone()
    }

    pub fn base_url(&self) -> String {
        let inner = self.inner.read().unwrap();

        inner.base_url.clone()
    }

    pub fn logout(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.authorization.delete();
    }

    pub(crate) async fn send(&self, request: Request) -> Result<Response> {
        self.client.send(request).await
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub(crate) struct ClientInner {
    base_url: String,
    authorization: Authorization,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("bad status code: {0}")]
    BadStatusCode(StatusCode),
    #[error("unauthorized")]
    Unauthorized,
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error("invalid token")]
    InvalidToken,
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authorization {
    pub(crate) tokens: Option<TokenPair>,
}

impl Authorization {
    pub fn new() -> Self {
        #[cfg(feature = "local-storage")]
        if let Ok(this) = LocalStorage::get("dynamic-tournament-api-client") {
            return this;
        }

        Self { tokens: None }
    }

    /// Update the authorization tokens to the newly provided [`TokenPair`].
    pub fn update(&mut self, tokens: TokenPair) {
        self.tokens = Some(tokens);

        #[cfg(feature = "local-storage")]
        {
            LocalStorage::set("dynamic-tournament-api-client", self)
                .expect("Failed to update localStorage with authorization credentials");
        }
    }

    /// Delete all authorization tokens.
    pub fn delete(&mut self) {
        #[cfg(feature = "local-storage")]
        LocalStorage::delete("dynamic-tournament-api-client");

        self.tokens = None;
    }

    /// Returns a reference to the auth token. This is the token to make requests. Returns [`None`]
    /// if no tokens are avaliable.
    pub fn auth_token(&self) -> Option<&str> {
        match self.tokens {
            Some(ref tokens) => Some(&tokens.auth_token),
            None => None,
        }
    }

    /// Returns a reference to the refresh token. Returns [`None`] if no tokens are avaliable.
    pub fn refresh_token(&self) -> Option<&str> {
        match self.tokens {
            Some(ref tokens) => Some(&tokens.refresh_token),
            None => None,
        }
    }
}

impl Default for Authorization {
    fn default() -> Self {
        Self::new()
    }
}
