mod payload;

pub mod auth;
pub mod http;
pub mod tournament;
pub mod v3;
pub mod websocket;

pub use payload::Payload;

use crate::http::{Request, RequestBuilder, Response};
use crate::tournament::TournamentClient;

use ::http::StatusCode;
use auth::{Token, TokenPair};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::borrow::Cow;
use std::sync::{Arc, RwLock};

#[cfg(feature = "local-storage")]
use gloo_storage::{LocalStorage, Storage};

#[cfg(target_family = "wasm")]
use gloo_utils::errors::JsError;

/// The primary client for the API.
#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<RwLock<ClientInner>>,
    client: http::Client,
}

impl Client {
    /// Creates a new `Client` with the given `base_url`.
    pub fn new<T>(base_url: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let inner = ClientInner {
            base_url: base_url.into(),
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

    pub(crate) fn request(&self) -> RequestBuilder {
        let inner = self.inner.read().unwrap();

        RequestBuilder::new(inner.base_url.to_string(), &inner.authorization)
    }

    /// Returns `true` if the `Client` has authentication credentials set.
    pub fn is_authenticated(&self) -> bool {
        let inner = self.inner.read().unwrap();
        inner.authorization.tokens.is_some()
    }

    pub fn authorization(&self) -> Authorization {
        let inner = self.inner.read().unwrap();

        inner.authorization.clone()
    }

    /// Returns the `base_url` used by the `Client`.
    pub fn base_url(&self) -> String {
        let inner = self.inner.read().unwrap();

        inner.base_url.to_string()
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
    base_url: Cow<'static, str>,
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
    #[cfg(target_family = "wasm")]
    #[error("JsError: {0}")]
    JsError(JsError),
}

// Manual impl required because JsError does not implement StdError.
#[cfg(target_family = "wasm")]
impl From<JsError> for Error {
    fn from(err: JsError) -> Self {
        Self::JsError(err)
    }
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
    #[inline]
    pub fn auth_token(&self) -> Option<&Token> {
        self.tokens.as_ref().map(|tokens| &tokens.auth_token)
    }

    /// Returns a reference to the refresh token. Returns [`None`] if no tokens are avaliable.
    #[inline]
    pub fn refresh_token(&self) -> Option<&Token> {
        self.tokens.as_ref().map(|tokens| &tokens.refresh_token)
    }
}

impl Default for Authorization {
    fn default() -> Self {
        Self::new()
    }
}
