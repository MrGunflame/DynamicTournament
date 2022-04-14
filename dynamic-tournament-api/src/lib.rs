pub mod auth;
pub mod tournament;

use crate::auth::AuthClient;
use crate::tournament::TournamentClient;

use reqwasm::http::{Headers, Method, Request};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::fmt::Write;
use std::sync::{Arc, RwLock};

#[cfg(feature = "local-storage")]
use gloo_storage::{LocalStorage, Storage};

#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<RwLock<ClientInner>>,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        let inner = ClientInner {
            base_url,
            authorization: Authorization::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
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
        inner.authorization.header.is_some()
    }

    pub fn logout(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.authorization.delete();
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub(crate) struct ClientInner {
    base_url: String,
    authorization: Authorization,
}

pub struct RequestBuilder {
    url: String,
    method: Method,
    headers: Vec<(&'static str, String)>,
    body: Option<String>,
}

impl RequestBuilder {
    fn new(url: String, authorization: &Authorization) -> Self {
        let this = Self {
            url,
            method: Method::GET,
            headers: Vec::new(),
            body: None,
        };

        match &authorization.header {
            Some(auth) => this.header("Authorization", auth),
            None => this,
        }
    }

    pub fn url<T>(mut self, url: T) -> Self
    where
        T: AsRef<str>,
    {
        write!(self.url, "{}", url.as_ref()).unwrap();
        self
    }

    pub fn get(mut self) -> Self {
        self.method = Method::GET;
        self
    }

    pub fn post(mut self) -> Self {
        self.method = Method::POST;
        self
    }

    pub fn put(mut self) -> Self {
        self.method = Method::PUT;
        self
    }

    pub fn header<T>(mut self, key: &'static str, val: T) -> Self
    where
        T: ToString,
    {
        self.headers.push((key, val.to_string()));
        self
    }

    pub fn body<T>(mut self, body: &T) -> Self
    where
        T: Serialize,
    {
        self.body = Some(serde_json::to_string(&body).unwrap());
        self.header("Content-Type", "application/json")
    }

    pub fn build(self) -> Request {
        let headers = Headers::new();
        for (key, val) in self.headers.into_iter() {
            headers.append(key, &val);
        }

        let mut req = Request::new(&self.url).method(self.method).headers(headers);

        if let Some(body) = self.body {
            req = req.body(body);
        }

        req
    }
}

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("bad status code: {0}")]
    BadStatusCode(u16),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Authorization {
    header: Option<String>,
}

impl Authorization {
    pub fn new() -> Self {
        let mut this = Self { header: None };

        #[cfg(feature = "local-storage")]
        if let Ok(new) = LocalStorage::get("dynamic-tournament-api-client") {
            this = new;
        }

        this
    }

    pub fn update<T>(&mut self, header: Option<T>)
    where
        T: ToString,
    {
        self.header = header.and_then(|v| Some(v.to_string()));

        #[cfg(feature = "local-storage")]
        {
            LocalStorage::set("dynamic-tournament-api-client", self)
                .expect("Failed to update localStorage with authorization credentials");
        }
    }

    pub fn delete(&mut self) {
        self.header = None;
    }
}

impl Default for Authorization {
    fn default() -> Self {
        Self::new()
    }
}
