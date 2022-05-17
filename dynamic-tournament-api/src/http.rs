use crate::{Authorization, Result};

use http::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Method, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};

use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub struct Error {
    #[cfg(any(target_family = "unix", target_family = "windows"))]
    #[from]
    error: hyper::Error,
    #[cfg(target_family = "wasm")]
    #[from]
    error: reqwasm::Error,
}

#[derive(Clone, Debug, Default)]
pub struct Client {
    #[cfg(any(target_family = "unix", target_family = "windows"))]
    inner: unix::InnerClient,
    #[cfg(target_family = "wasm")]
    inner: wasm::InnerClient,
}

impl Client {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn send(&self, request: Request) -> Result<Response> {
        self.inner.send(request).await
    }
}

#[derive(Clone, Debug)]
pub struct Request {
    uri: String,
    method: Method,
    headers: Vec<(&'static str, String)>,
    body: Option<String>,
}

impl Request {
    pub fn builder() -> RequestBuilder {
        RequestBuilder::default()
    }
}

impl Default for Request {
    fn default() -> Self {
        Self {
            uri: String::new(),
            method: Method::GET,
            headers: Vec::new(),
            body: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RequestBuilder {
    inner: Request,
}

impl RequestBuilder {
    pub fn new(uri: String, authorization: &Authorization) -> Self {
        let mut inner = Request {
            uri,
            ..Default::default()
        };

        if let Some(token) = authorization.auth_token() {
            inner
                .headers
                .push((AUTHORIZATION.as_str(), format!("Bearer {}", token)))
        }

        Self { inner }
    }

    /// Sets the request method to `OPTIONS`.
    pub fn options(mut self) -> Self {
        self.inner.method = Method::OPTIONS;
        self
    }

    /// Sets the request method to `GET`.
    pub fn get(mut self) -> Self {
        self.inner.method = Method::GET;
        self
    }

    /// Sets the request method to `POST`.
    pub fn post(mut self) -> Self {
        self.inner.method = Method::POST;
        self
    }

    /// Sets the request method to `PUT`.
    pub fn put(mut self) -> Self {
        self.inner.method = Method::PUT;
        self
    }

    /// Sets the request method to `DELETE`.
    pub fn delete(mut self) -> Self {
        self.inner.method = Method::DELETE;
        self
    }

    /// Sets the request method to `PATCH`.
    pub fn patch(mut self) -> Self {
        self.inner.method = Method::PATCH;
        self
    }

    pub fn uri(mut self, uri: &str) -> Self {
        self.inner.uri.push_str(uri);
        self
    }

    /// Adds an header to the request.
    pub fn header<T>(mut self, key: &'static str, value: T) -> Self
    where
        T: ToString,
    {
        self.inner.headers.push((key, value.to_string()));
        self
    }

    /// Uses `T` serialized as json as the request body.
    pub fn body<T>(mut self, body: &T) -> Self
    where
        T: Serialize,
    {
        self.inner.body = Some(serde_json::to_string(&body).unwrap());
        self.header(CONTENT_TYPE.as_str(), "application/json")
    }

    pub fn build(self) -> Request {
        self.inner
    }
}

impl From<RequestBuilder> for Request {
    fn from(req: RequestBuilder) -> Self {
        req.inner
    }
}

#[derive(Debug)]
pub struct Response {
    #[cfg(any(target_family = "unix", target_family = "windows"))]
    inner: unix::InnerResponse,
    #[cfg(target_family = "wasm")]
    inner: wasm::InnerResponse,
}

impl Response {
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Returns `true` if the response contains a 2xx status code.
    pub fn is_success(&self) -> bool {
        self.status().is_success()
    }

    pub async fn json<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.inner.json().await
    }
}

#[cfg(any(target_family = "unix", target_family = "windows"))]
mod unix {
    use super::{Error, Request, Response};
    use crate::Result;

    use http::StatusCode;
    use hyper::{body, client::HttpConnector, Body};
    use hyper_tls::HttpsConnector;
    use serde::de::DeserializeOwned;

    #[derive(Clone, Debug)]
    pub struct InnerClient {
        inner: hyper::Client<HttpsConnector<HttpConnector>>,
    }

    impl InnerClient {
        pub async fn send(&self, request: Request) -> Result<Response> {
            let req = request.into();

            let resp = self.inner.request(req).await.map_err(Error::from)?;

            Ok(Response {
                inner: InnerResponse(resp),
            })
        }
    }

    impl Default for InnerClient {
        fn default() -> Self {
            Self {
                inner: hyper::Client::builder().build(HttpsConnector::new()),
            }
        }
    }

    #[derive(Debug)]
    pub struct InnerResponse(hyper::Response<Body>);

    impl InnerResponse {
        pub fn status(&self) -> StatusCode {
            self.0.status()
        }

        pub async fn json<T>(self) -> Result<T>
        where
            T: DeserializeOwned,
        {
            let bytes = body::to_bytes(self.0.into_body())
                .await
                .map_err(Error::from)?;

            Ok(serde_json::from_slice(&bytes)?)
        }
    }

    impl From<Request> for hyper::Request<Body> {
        fn from(request: Request) -> Self {
            let body = match request.body {
                Some(body) => Body::from(body),
                None => Body::empty(),
            };

            let mut builder = hyper::Request::builder()
                .uri(request.uri)
                .method(request.method);

            for (key, value) in request.headers {
                builder = builder.header(key, value);
            }

            builder.body(body).unwrap()
        }
    }
}

#[cfg(target_family = "wasm")]
mod wasm {
    use super::{Error, Request, Response};
    use crate::Result;

    use http::{Method, StatusCode};
    use serde::de::DeserializeOwned;

    #[derive(Copy, Clone, Debug, Default)]
    pub struct InnerClient;

    impl InnerClient {
        pub async fn send(&self, request: Request) -> Result<Response> {
            let mut req = reqwasm::http::Request::new(&request.uri).method(match request.method {
                Method::OPTIONS => reqwasm::http::Method::OPTIONS,
                Method::GET => reqwasm::http::Method::GET,
                Method::POST => reqwasm::http::Method::POST,
                Method::PUT => reqwasm::http::Method::PUT,
                Method::DELETE => reqwasm::http::Method::DELETE,
                Method::PATCH => reqwasm::http::Method::PATCH,
                _ => unreachable!(),
            });

            if let Some(body) = request.body {
                req = req.body(body);
            }

            let resp = req.send().await.map_err(Error::from)?;

            Ok(Response {
                inner: InnerResponse(resp),
            })
        }
    }

    #[derive(Debug)]
    pub struct InnerResponse(reqwasm::http::Response);

    impl InnerResponse {
        pub fn status(&self) -> StatusCode {
            StatusCode::from_u16(self.0.status()).unwrap()
        }

        pub async fn json<T>(self) -> Result<T>
        where
            T: DeserializeOwned,
        {
            Ok(self.0.json().await.map_err(Error::from)?)
        }
    }
}
