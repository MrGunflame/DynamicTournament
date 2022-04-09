pub mod auth;
pub mod tournament;

use crate::tournament::TournamentClient;

use reqwasm::http::{Headers, Method, Request};
use serde::Serialize;
use thiserror::Error;

use std::fmt::Write;
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct Client {
    inner: Arc<RwLock<ClientInner>>,
}

impl Client {
    pub fn new(base_url: String) -> Self {
        let inner = ClientInner {
            base_url,
            authorization: None,
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }

    pub fn tournaments(&self) -> TournamentClient<'_> {
        TournamentClient::new(self)
    }

    pub(crate) fn request(&self) -> RequestBuilder {
        let inner = self.inner.read().unwrap();

        RequestBuilder::new(inner.base_url.clone(), inner.authorization.clone())
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug)]
pub(crate) struct ClientInner {
    base_url: String,
    authorization: Option<String>,
}

pub struct RequestBuilder {
    url: String,
    method: Method,
    headers: Vec<(&'static str, String)>,
    body: String,
}

impl RequestBuilder {
    fn new(url: String, authorization: Option<String>) -> Self {
        let this = Self {
            url,
            method: Method::GET,
            headers: Vec::new(),
            body: String::new(),
        };

        match authorization {
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
        self.body = serde_json::to_string(&body).unwrap();
        self.header("Content-Type", "application/json")
    }

    pub fn build(self) -> Request {
        let headers = Headers::new();
        for (key, val) in self.headers.into_iter() {
            headers.append(key, &val);
        }

        Request::new(&self.url)
            .method(self.method)
            .headers(headers)
            .body(self.body)
    }
}

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("bad status code: {0}")]
    BadStatusCode(u16),
}
