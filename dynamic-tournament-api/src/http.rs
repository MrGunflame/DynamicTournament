use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::{Method, StatusCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

use crate::{Authorization, Result};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error(transparent)]
    Status(#[from] StatusError),
}

/// An error from the server, rejecting the connection.
#[derive(Clone, Debug, Error)]
#[error("status code {status}")]
pub struct StatusError {
    status: StatusCode,
}

impl StatusError {
    /// Creates a new `StatusError` from the response.
    #[inline]
    fn new(resp: Response) -> Self {
        Self {
            status: resp.status(),
        }
    }

    /// Returns the [`StatusCode`] of the error.
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Returns the status code of the error as an `u16`.
    #[inline]
    pub fn status_u16(&self) -> u16 {
        self.status.as_u16()
    }
}

/// An error in the http protocol.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct HttpError {
    #[cfg(not(target_family = "wasm"))]
    #[from]
    error: sys::Error,

    #[cfg(target_family = "wasm")]
    #[from]
    error: wasm::Error,
}

#[derive(Clone, Debug, Default)]
pub struct Client {
    #[cfg(not(target_family = "wasm"))]
    inner: sys::Client,

    #[cfg(target_family = "wasm")]
    inner: wasm::Client,
}

impl Client {
    /// Creates a new `Client`.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sends a given [`Request`] using the `Client`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] when sending the request fails or the server responds with
    /// a non-2xx status code.
    ///
    /// [`Error`]: enum@crate::Error
    pub async fn send(&self, request: Request) -> Result<Response> {
        log::debug!("Sending {}", request.uri);

        let resp = self.inner.send(request).await?;

        log::debug!("Read status {}", resp.status());

        if resp.is_success() {
            Ok(resp)
        } else {
            Err(Error::Status(StatusError::new(resp)).into())
        }
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
    /// Creates a new [`RequestBuilder`].
    #[inline]
    pub fn builder() -> RequestBuilder {
        RequestBuilder::default()
    }
}

impl Default for Request {
    #[inline]
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
    #[inline]
    pub fn options(mut self) -> Self {
        self.inner.method = Method::OPTIONS;
        self
    }

    /// Sets the request method to `GET`.
    #[inline]
    pub fn get(mut self) -> Self {
        self.inner.method = Method::GET;
        self
    }

    /// Sets the request method to `POST`.
    #[inline]
    pub fn post(mut self) -> Self {
        self.inner.method = Method::POST;
        self
    }

    /// Sets the request method to `PUT`.
    #[inline]
    pub fn put(mut self) -> Self {
        self.inner.method = Method::PUT;
        self
    }

    /// Sets the request method to `DELETE`.
    #[inline]
    pub fn delete(mut self) -> Self {
        self.inner.method = Method::DELETE;
        self
    }

    /// Sets the request method to `PATCH`.
    #[inline]
    pub fn patch(mut self) -> Self {
        self.inner.method = Method::PATCH;
        self
    }

    #[inline]
    pub fn uri(mut self, uri: &str) -> Self {
        self.inner.uri.push_str(uri);
        self
    }

    /// Adds an header to the request.
    #[inline]
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

    /// Consumes the builder, returning the built [`Request`].
    #[inline]
    pub fn build(self) -> Request {
        self.inner
    }
}

impl From<RequestBuilder> for Request {
    #[inline]
    fn from(req: RequestBuilder) -> Self {
        req.inner
    }
}

#[derive(Debug)]
pub struct Response {
    #[cfg(not(target_family = "wasm"))]
    inner: sys::InnerResponse,
    #[cfg(target_family = "wasm")]
    inner: wasm::InnerResponse,
}

impl Response {
    /// Returns the [`StatusCode`] of the response.
    pub fn status(&self) -> StatusCode {
        self.inner.status()
    }

    /// Returns `true` if the response contains a 2xx status code.
    pub fn is_success(&self) -> bool {
        self.status().is_success()
    }

    /// Parses the body of the response as json.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if reading the body fails or the body contains invalid json.
    ///
    /// [`Error`]: enum@crate::Error
    pub async fn json<T>(self) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.inner.json().await
    }
}

// System http implementation (for non-wasm targets)
#[cfg(not(target_family = "wasm"))]
mod sys {
    use super::{Request, Response};
    use crate::Result;

    use http::StatusCode;
    use hyper::{body, client::HttpConnector, Body};
    use hyper_tls::HttpsConnector;
    use serde::de::DeserializeOwned;

    use super::HttpError;

    pub use hyper::Error;

    #[derive(Clone, Debug)]
    pub struct Client {
        inner: hyper::Client<HttpsConnector<HttpConnector>>,
    }

    impl Client {
        pub fn new() -> Self {
            Self {
                inner: hyper::Client::builder().build(HttpsConnector::new()),
            }
        }

        pub async fn send(&self, req: Request) -> Result<Response> {
            let req = convert_request(req);

            let resp = match self.inner.request(req).await {
                Ok(resp) => resp,
                Err(err) => return Err(super::Error::Http(HttpError::from(err)).into()),
            };

            Ok(Response {
                inner: InnerResponse(resp),
            })
        }
    }

    impl Default for Client {
        #[inline]
        fn default() -> Self {
            Self::new()
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
            let bytes = match body::to_bytes(self.0.into_body()).await {
                Ok(bytes) => bytes,
                Err(err) => return Err(super::Error::Http(HttpError::from(err)).into()),
            };

            Ok(serde_json::from_slice(&bytes)?)
        }
    }

    fn convert_request(req: Request) -> hyper::Request<Body> {
        let body = match req.body {
            Some(body) => Body::from(body),
            None => Body::empty(),
        };

        let mut builder = hyper::Request::builder().uri(req.uri).method(req.method);

        for (key, val) in req.headers {
            builder = builder.header(key, val);
        }

        builder.body(body).unwrap()
    }
}

#[cfg(target_family = "wasm")]
mod wasm {
    use super::{Request, Response};
    use crate::Result;

    use http::{Method, StatusCode};
    use serde::de::DeserializeOwned;

    use super::HttpError;

    pub use reqwasm::Error;

    #[derive(Copy, Clone, Debug, Default)]
    pub struct Client;

    impl Client {
        pub async fn send(&self, req: Request) -> Result<Response> {
            let req = convert_request(req);

            let resp = match req.send().await {
                Ok(resp) => resp,
                Err(err) => return Err(super::Error::Http(HttpError::from(err)).into()),
            };

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
            match self.0.json().await {
                Ok(val) => Ok(val),
                Err(err) => Err(super::Error::Http(HttpError::from(err)).into()),
            }
        }
    }

    fn convert_request(req: Request) -> reqwasm::http::Request {
        let mut builder = reqwasm::http::Request::new(&req.uri);
        builder = builder.method(match req.method {
            Method::OPTIONS => reqwasm::http::Method::OPTIONS,
            Method::GET => reqwasm::http::Method::GET,
            Method::POST => reqwasm::http::Method::POST,
            Method::PUT => reqwasm::http::Method::PUT,
            Method::DELETE => reqwasm::http::Method::DELETE,
            Method::PATCH => reqwasm::http::Method::PATCH,
            _ => unreachable!(),
        });

        for (key, val) in req.headers {
            builder = builder.header(key, &val);
        }

        if let Some(body) = req.body {
            builder = builder.body(body);
        }

        builder
    }
}
