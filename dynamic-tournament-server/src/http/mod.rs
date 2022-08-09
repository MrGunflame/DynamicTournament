mod v1;
pub mod v2;
mod v3;

#[cfg(feature = "metrics")]
mod metrics;

use crate::config::BindAddr;
use crate::{Error, State, StatusCodeError};

use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::future::BoxFuture;
use futures::Future;
use hyper::header::{
    HeaderValue, IntoHeaderName, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN,
    AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE,
};
use hyper::http::request::Parts;
use hyper::server::conn::Http;
use hyper::service::Service;
use hyper::{Body, HeaderMap, Method, StatusCode, Uri};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpSocket;
use tokio::time::Instant;

#[cfg(target_family = "unix")]
use {std::path::Path, tokio::net::UnixListener};

pub type Result = std::result::Result<Response, Error>;

pub async fn bind(addr: BindAddr, state: State) -> std::result::Result<(), crate::Error> {
    match addr {
        BindAddr::Tcp(addr) => bind_tcp(addr, state).await,
        #[cfg(target_family = "unix")]
        BindAddr::Unix(path) => bind_unix(path, state).await,
        #[cfg(not(target_family = "unix"))]
        BindAddr::Unix(_) => panic!("Cannot bind to unix socket on non-unix target"),
    }
}

async fn bind_tcp(addr: SocketAddr, state: State) -> std::result::Result<(), crate::Error> {
    let mut shutdown = state.shutdown.listen();

    let socket = TcpSocket::new_v4()?;
    if let Err(err) = socket.set_reuseaddr(true) {
        log::warn!("Failed to set SO_REUSEADDR flag: {}", err);
    }

    // Enable SO_REUSEPORT for all supported systems.
    #[cfg(all(unix, not(target_os = "solaris"), not(target_os = "illumos")))]
    if let Err(err) = socket.set_reuseport(true) {
        log::warn!("Failed to set SO_REUSEPORT flag: {}", err);
    }

    socket.bind(addr)?;
    let listener = socket.listen(1024)?;
    log::info!("Server running on {}", addr);
    loop {
        tokio::select! {
            res = listener.accept() => {
                let (stream, addr) = match res {
                    Ok((stream, addr)) => (stream, addr),
                    Err(err) => {
                        log::warn!("Failed to accept connection: {:?}", err);
                        continue;
                    }
                };
                log::info!("Accepting new connection from {:?}", addr);

                let state = state.clone();
                tokio::task::spawn(async move {
                    serve_connection(stream, state).await;
                });
            }
            // Shut down the server.
            _ = &mut shutdown => {
                log::debug!("Shutting down http server");
                return Ok(());
            }
        }
    }
}

/// Binds a new HTTP server to a unix socket.
///
/// Note that `bind_unix` is only avaliable on unix targets.
#[cfg(target_family = "unix")]
async fn bind_unix<P>(path: P, state: State) -> std::result::Result<(), crate::Error>
where
    P: AsRef<Path>,
{
    let mut shutdown = state.shutdown.listen();
    let path = path.as_ref();

    let listener = UnixListener::bind(path)?;
    log::debug!("Server running on {}", path.display());
    loop {
        tokio::select! {
            res = listener.accept() => {
                let (stream, addr) = match res {
                    Ok((stream, addr)) => (stream, addr),
                    Err(err) => {
                        log::warn!("Failed to accept connection: {:?}", err);
                        continue;
                    }
                };

                log::info!("Accepting new connection from {:?}", addr);

                let state = state.clone();
                tokio::task::spawn(async move {
                    serve_connection(stream, state).await;
                });
            }
            _ = &mut shutdown => {
                log::debug!("Shutting down http server");

                // Remove the unix socket.
                tokio::fs::remove_file(path).await?;

                return Ok(());
            }
        }
    }
}

async fn serve_connection<S>(stream: S, state: State)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    let shutdown = state.shutdown.listen();
    let service = RootService { state };

    let mut conn = Http::new()
        .http1_keep_alive(true)
        .serve_connection(stream, service)
        .with_upgrades();

    tokio::select! {
        res = &mut conn => {
            if let Err(err) = res {
                log::warn!("Error serving HTTP conn: {}", err);
            }
        }
        _ = shutdown => {
            log::trace!("HTTP conn shutdown");
            Pin::new(&mut conn).graceful_shutdown();

            if let Err(err) = conn.await {
                log::warn!("Error serving HTTP conn: {}", err);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct RootService {
    state: State,
}

impl Service<hyper::Request<Body>> for RootService {
    type Response = hyper::Response<Body>;
    type Error = crate::Error;
    type Future = RootServiceFuture;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        RootServiceFuture::new(req, self.state.clone())
    }
}

struct RootServiceFuture(
    BoxFuture<'static, std::result::Result<hyper::Response<Body>, crate::Error>>,
);

impl RootServiceFuture {
    fn new(req: hyper::Request<Body>, state: State) -> Self {
        Self(Box::pin(async move {
            Ok(service_root(req, state).await.unwrap())
        }))
    }
}

impl Future for RootServiceFuture {
    type Output = std::result::Result<hyper::Response<Body>, crate::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let future = unsafe { self.map_unchecked_mut(|this| &mut this.0) };
        future.poll(cx)
    }
}

async fn service_root(
    req: hyper::Request<Body>,
    state: State,
) -> std::result::Result<hyper::Response<Body>, Infallible> {
    log::trace!("Received Request:");
    log::trace!("Head: {} {}", req.method(), req.uri());
    log::trace!("Headers: {:?}", req.headers());
    log::trace!("Body: {:?}", req.body());

    #[cfg(feature = "metrics")]
    state.metrics.http_requests_total.inc();

    let req = Request::new(req, state);

    let uri = String::from(req.uri().path());

    let mut uri = RequestUri::new(&uri);

    log::debug!("{:?}", uri);

    let origin = req.headers().get("Origin").cloned();

    let res = match uri.take_str() {
        Some("v1") => v1::route().await,
        Some("v2") => v2::route(req, uri).await,
        Some("v3") => v3::route(req, uri).await,
        #[cfg(feature = "metrics")]
        Some("metrics") => metrics::route(req).await,
        _ => Err(Error::NotFound),
    };

    match res {
        Ok(mut resp) => {
            log::debug!("Settings CORS for origin: {:?}", origin);
            if let Some(origin) = origin {
                resp = resp.header(ACCESS_CONTROL_ALLOW_ORIGIN, origin);
            }

            resp = resp.header(
                ACCESS_CONTROL_ALLOW_HEADERS,
                HeaderValue::from_static("content-type,authorization"),
            );

            Ok(resp.build())
        }
        Err(err) => {
            let mut resp = Response::ok();

            match err {
                Error::NotFound => {
                    resp = resp.status(StatusCode::NOT_FOUND).body("Not Found");
                }
                Error::BadRequest => {
                    resp = resp.status(StatusCode::BAD_REQUEST).body("Bad Request");
                }
                Error::MethodNotAllowed => {
                    resp = resp
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body("Method Not Allowed");
                }
                Error::StatusCodeError(err) => {
                    log::debug!("Responding with error: {:?}", err);

                    resp = resp.status(err.code).json(&ErrorResponse {
                        code: err.code.as_u16(),
                        message: err.message,
                    });
                }
                err => {
                    log::error!("{:?}", err);

                    resp = resp
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Internal Server Error");
                }
            }

            Ok(resp.build())
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub parts: Parts,
    pub body: Option<Body>,
    state: State,
}

impl Request {
    const BODY_MAX_SIZE: usize = 16384;

    #[inline]
    fn new(req: hyper::Request<Body>, state: State) -> Self {
        let (parts, body) = req.into_parts();

        Self {
            parts,
            body: Some(body),
            state,
        }
    }

    #[inline]
    pub fn state(&self) -> &State {
        &self.state
    }

    #[inline]
    pub fn method(&self) -> &Method {
        &self.parts.method
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        &self.parts.headers
    }

    #[inline]
    pub fn uri(&self) -> &Uri {
        &self.parts.uri
    }

    /// Aggregates and returns the whole request body once it was fully recieved.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] when reading the request body fails or the request times out.
    pub async fn body(&mut self) -> std::result::Result<hyper::body::Bytes, Error> {
        // Check if the "Content-Length" header is valid.
        if self.content_length()? > Self::BODY_MAX_SIZE {
            return Err(StatusCodeError::payload_too_large().into());
        }

        const DUR: Duration = Duration::new(30, 0);

        let deadline = Instant::now() + DUR;

        let body = self.body.take().ok_or(Error::BodyConsumed)?;
        tokio::select! {
            res = hyper::body::to_bytes(body) => {
                Ok(res?)
            }
            _ = tokio::time::sleep_until(deadline) => {
                log::info!("Client failed to transmit body in {}s, dropping connection", DUR.as_secs());
                Err(StatusCodeError::request_timeout().into())
            }
        }
    }

    /// Aggregates and returns the whole request body parsed as json once it was fully received.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] when reading the request body fails, the request times out or the
    /// body contains an invalid json payload.
    pub async fn json<T>(&mut self) -> std::result::Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let bytes = self.body().await?;
        match serde_json::from_slice(&bytes) {
            Ok(value) => Ok(value),
            Err(err) => {
                log::debug!("Failed to read request body as json: {}", err);

                Err(StatusCodeError::new(StatusCode::BAD_REQUEST, err).into())
            }
        }
    }

    /// Returns the value of the "Content-Length" header. If the header is not present or has an
    /// invalid value an error is returned.
    pub fn content_length(&self) -> std::result::Result<usize, Error> {
        match self.headers().get(CONTENT_LENGTH) {
            Some(value) => match value.to_str() {
                Ok(value) => match value.parse() {
                    Ok(value) => Ok(value),
                    Err(err) => {
                        log::debug!("Failed to parse \"Content-Length\" header: {:?}", err);

                        Err(StatusCodeError::bad_request().into())
                    }
                },
                Err(err) => {
                    log::debug!("Failed to parse \"Content-Length\" header: {:?}", err);

                    Err(StatusCodeError::bad_request().into())
                }
            },
            None => Err(StatusCodeError::length_required().into()),
        }
    }

    /// Returns the value of the `Authorization` header. Returns an [`enum@Error`] if the header is
    /// missing or contains a non-string value.
    ///
    /// # Errors
    ///
    /// Returns [`StatusCodeError::unauthorized`] if the header is missing. Returns
    /// [`StatusCodeError::bad_request`] if the header contains a non-string value.
    pub fn authorization(&self) -> std::result::Result<&str, Error> {
        match self.headers().get(AUTHORIZATION) {
            Some(val) => match val.to_str() {
                Ok(val) => Ok(val),
                Err(_) => Err(StatusCodeError::bad_request().into()),
            },
            None => Err(StatusCodeError::unauthorized().into()),
        }
    }

    /// Asserts that the request is authenticated. Returns an [`enum@Error`] if this is not the case.
    pub fn require_authentication(&self) -> std::result::Result<(), Error> {
        let header = self.authorization()?;

        let mut parts = header.split(' ');

        match parts.next() {
            Some("Bearer") => (),
            _ => return Err(StatusCodeError::bad_request().into()),
        }

        let token = match parts.next() {
            Some(token) => token,
            None => return Err(StatusCodeError::bad_request().into()),
        };

        match self.state().auth.validate_auth_token(token) {
            Ok(_) => Ok(()),
            Err(_) => Err(StatusCodeError::unauthorized().into()),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RequestUri<'a> {
    path: &'a str,
}

impl<'a> RequestUri<'a> {
    pub fn new(mut path: &'a str) -> Self {
        if path.starts_with('/') {
            path = &path[1..];
        }

        Self { path }
    }

    pub fn take(&mut self) -> Option<UriPart> {
        let part = self.take_str()?;

        let part = UriPart { part };

        Some(part)
    }

    pub fn take_str(&mut self) -> Option<&str> {
        if self.path.is_empty() {
            None
        } else {
            Some(match self.path.split_once('/') {
                Some((part, rem)) => {
                    self.path = rem;
                    part
                }
                None => {
                    let path = self.path;
                    self.path = "";
                    path
                }
            })
        }
    }

    pub fn take_all(self) -> Option<&'a str> {
        if self.path.is_empty() {
            None
        } else {
            Some(self.path)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UriPart<'a> {
    part: &'a str,
}

impl<'a> UriPart<'a> {
    pub fn parse<T>(&self) -> std::result::Result<T, Error>
    where
        T: FromStr,
    {
        match self.part.parse() {
            Ok(v) => Ok(v),
            Err(_) => Err(StatusCodeError::not_found().into()),
        }
    }
}

impl<'a> AsRef<str> for UriPart<'a> {
    fn as_ref(&self) -> &str {
        self.part
    }
}

impl<'a> PartialEq<str> for UriPart<'a> {
    fn eq(&self, other: &str) -> bool {
        self.part == other
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: u16,
    pub message: String,
}

#[derive(Debug)]
pub struct Response {
    status: StatusCode,
    headers: HeaderMap,
    body: Body,
}

impl Response {
    /// 101 Switching Protocols
    pub fn switching_protocols() -> Self {
        Self {
            status: StatusCode::SWITCHING_PROTOCOLS,
            headers: HeaderMap::new(),
            body: Body::empty(),
        }
    }

    /// 200 OK
    pub fn ok() -> Self {
        Self {
            status: StatusCode::OK,
            headers: HeaderMap::new(),
            body: Body::empty(),
        }
    }

    /// 201 Created
    pub fn created() -> Self {
        Self {
            status: StatusCode::CREATED,
            headers: HeaderMap::new(),
            body: Body::empty(),
        }
    }

    /// 204 No Content
    pub fn no_content() -> Self {
        Self {
            status: StatusCode::NO_CONTENT,
            headers: HeaderMap::new(),
            body: Body::empty(),
        }
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn body<T>(mut self, body: T) -> Self
    where
        T: Into<Body>,
    {
        self.body = body.into();
        self
    }

    pub fn json<T>(mut self, body: &T) -> Self
    where
        T: Serialize,
    {
        self.body = Body::from(serde_json::to_vec(body).unwrap());
        self.header(CONTENT_TYPE, HeaderValue::from_static("application/json"))
    }

    pub fn header<K>(mut self, key: K, value: HeaderValue) -> Self
    where
        K: IntoHeaderName,
    {
        self.headers.append(key, value);
        self
    }

    fn build(self) -> hyper::Response<Body> {
        let mut resp = hyper::Response::new(self.body);
        *resp.status_mut() = self.status;
        *resp.headers_mut() = self.headers;
        resp
    }
}

/// Checks the request method and runs the specified path. If no matching method is found
/// an method_not_allowed error is returned.
#[macro_export]
macro_rules! method {
    ($req:expr, {$($method:expr => $branch:expr),* $(,)?}) => {
        match $req.method() {
            $(
                method if method == $method => $branch,
            )*
            method if method == hyper::Method::OPTIONS => {
                use $crate::http::Response;
                use hyper::header::{HeaderValue, ALLOW, ACCESS_CONTROL_ALLOW_METHODS};

                let allow = vec![$($method.as_str()),*];
                let allow = HeaderValue::from_bytes(allow.join(",").as_bytes()).unwrap();

                Ok(Response::no_content()
                    .header(ALLOW, allow.clone())
                    .header(ACCESS_CONTROL_ALLOW_METHODS,allow))
            }
            _ => Err($crate::StatusCodeError::method_not_allowed().into()),
        }
    };
}
