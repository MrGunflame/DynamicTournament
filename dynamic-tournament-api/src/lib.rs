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
use gloo_storage::{LocalStorage, Storage as _};

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
        inner.authorization.auth_token().is_some()
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
    auth_token: Option<Token>,
    refresh_token: Option<Token>,
}

impl Authorization {
    pub fn new() -> Self {
        let refresh_token = Storage::load()
            .map(|storage| storage.token)
            .and_then(|token| match Token::new(token) {
                Ok(token) => Some(token),
                Err(err) => {
                    log::warn!("Loaded token is invalid: {}", err);
                    None
                }
            });

        Self {
            auth_token: None,
            refresh_token,
        }
    }

    /// Update the authorization tokens to the newly provided [`TokenPair`].
    pub fn update(&mut self, tokens: TokenPair) {
        self.auth_token = Some(tokens.auth_token);

        let refresh_token = tokens.refresh_token.to_string();
        self.refresh_token = Some(tokens.refresh_token);

        Storage {
            token: refresh_token,
        }
        .update();
    }

    /// Delete all authorization tokens.
    pub fn delete(&mut self) {
        self.auth_token = None;
        self.refresh_token = None;

        Storage::delete();
    }

    /// Returns a reference to the auth token. This is the token to make requests. Returns [`None`]
    /// if no tokens are avaliable.
    #[inline]
    pub fn auth_token(&self) -> Option<&Token> {
        self.auth_token.as_ref()
    }

    /// Returns a reference to the refresh token. Returns [`None`] if no tokens are avaliable.
    #[inline]
    pub fn refresh_token(&self) -> Option<&Token> {
        self.refresh_token.as_ref()
    }
}

impl Default for Authorization {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Storage {
    /// Refresh token
    pub token: String,
}

impl Storage {
    // Used from within feature-gates.
    #[allow(unused)]
    const KEY: &'static str = "dynamic-tournament-api-client";

    pub fn load() -> Option<Self> {
        #[cfg(feature = "local-storage")]
        match LocalStorage::get(Self::KEY) {
            Ok(val) => Some(val),
            Err(err) => {
                log::warn!("Failed to load from storage: {}", err);
                None
            }
        }

        #[cfg(not(feature = "local-storage"))]
        None
    }

    pub fn update(&self) {
        #[cfg(feature = "local-storage")]
        if let Err(err) = LocalStorage::set(Self::KEY, self) {
            log::error!("Failed to save to storage: {}", err);
        }
    }

    pub fn delete() {
        #[cfg(feature = "local-storage")]
        LocalStorage::delete(Self::KEY);
    }
}
