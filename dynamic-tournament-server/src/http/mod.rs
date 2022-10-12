pub mod etag;
pub mod path;
mod v1;
pub mod v2;
pub mod v3;

#[cfg(feature = "metrics")]
mod metrics;

use crate::config::BindAddr;
use crate::limits::File;
use crate::{Error, State, StatusCodeError};

use std::convert::Infallible;
use std::mem;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{self, Poll};
use std::time::Duration;

use dynamic_tournament_api::auth::Flags;
use futures::future::BoxFuture;
use futures::Future;
use hyper::header::{
    HeaderValue, IntoHeaderName, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN,
    AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE, ETAG, IF_MATCH, IF_NONE_MATCH,
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

use self::etag::Etag;

#[cfg(target_family = "unix")]
use {std::path::Path, tokio::net::UnixListener};

pub use dynamic_tournament_macros::{method, path};

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
        let fut = listener.accept();

        if accept_connection(fut, &state).await.is_err() {
            return Ok(());
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
    use std::io::ErrorKind;

    let path = path.as_ref();

    // Bind to the socket. If the socket file already exists and bind() returns
    // EADDRINUSE remove it and try to bind again.
    let listener = match UnixListener::bind(path) {
        Ok(socket) => socket,
        Err(err) => {
            if err.kind() == ErrorKind::AddrInUse {
                log::warn!("The socket already exists, did the server crash?");
                tokio::fs::remove_file(path).await?;

                UnixListener::bind(path)?
            } else {
                return Err(err.into());
            }
        }
    };

    log::debug!("Server running on {}", path.display());
    loop {
        let fut = listener.accept();

        if accept_connection(fut, &state).await.is_err() {
            return Ok(());
        }
    }
}

/// Accepts a new connection from `F`. This function returns `Ok(())` if operation should continue
/// normally or `Err(())` if the server should abort.
async fn accept_connection<F, S, A>(fut: F, state: &State) -> std::result::Result<(), ()>
where
    F: Future<Output = std::io::Result<(S, A)>>,
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    A: std::fmt::Debug,
{
    let mut shutdown = state.shutdown.listen();

    tokio::select! {
        file = state.limits.acquire_file() => {
            tokio::select! {
                res = fut => {
                    let (stream, addr) = match res {
                        Ok(val) => val,
                        Err(err) => {
                            log::warn!("Failed to accept connection: {:?}", err);
                            return Ok(());
                        }
                    };

                    log::debug!("Accepting new connection from {:?}", addr);

                    // SAFETY: State always outlives `accept_connection`.
                    let file = unsafe { std::mem::transmute(file) };
                    let state = state.clone();
                    tokio::task::spawn(async move {
                        serve_connection(stream, state, file).await;
                    });
                }
                _ = &mut shutdown => {
                    return Err(());
                }
            }
        }
        _ = &mut shutdown => {
            return Err(());
        }
    }

    Ok(())
}

async fn serve_connection<S>(stream: S, state: State, file: File<'static>)
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
                if err.is_user() {
                    log::warn!("Error serving HTTP conn: {}", err);
                } else {
                    log::debug!("Error serving HTTP conn: {}", err);
                }

            }
        }
    }

    // Release the file descriptor.
    drop(file);
}

#[derive(Clone, Debug)]
struct RootService {
    state: State,
}

impl Service<hyper::Request<Body>> for RootService {
    type Response = hyper::Response<Body>;
    type Error = crate::Error;
    type Future = RootServiceFuture;

    fn poll_ready(
        &mut self,
        _cx: &mut task::Context,
    ) -> Poll<std::result::Result<(), Self::Error>> {
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

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Self::Output> {
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

    let mut ctx = Context::new(req, state);

    let origin = ctx.req.headers().get("Origin").cloned();

    let res = path!(ctx, {
        "v1" => v1::route().await,
        "v2" => v2::route(ctx).await,
        "v3" => v3::route(ctx).await,
        #[cfg(feature = "metrics")]
        "metrics" => metrics::route(ctx).await,
    });

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
                Error::StatusCodeError(err) => {
                    log::debug!("Responding with error: {:?}", err);

                    resp = resp.status(err.code).json(&ErrorResponse {
                        code: err.code.as_u16(),
                        message: err.message,
                    });
                }
                err => {
                    log::error!("HTTP handler returned an error: {}", err);
                    log::warn!("Responding with 500");

                    resp = resp
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(&ErrorResponse {
                            code: 500,
                            message: "Internal Server Error".into(),
                        });
                }
            }

            Ok(resp.build())
        }
    }
}

/// A context carried with a HTTP request.
pub struct Context {
    pub req: Request,
    pub state: State,
    /// This field is the owner of `self.path`. It is only ever accessed through that
    /// reference and therefore not dead code.
    _path_buf: Box<str>,
    /// A self-referential field borrowing `self._path_buf`. We need to make sure it is never
    /// made avaliable to the outside using the internal 'static lifetime. The field is only
    /// valid for as long as self is.
    path: path::Path<'static>,
}

impl Context {
    fn new(req: hyper::Request<Body>, state: State) -> Self {
        let req = Request::new(req);

        // We make sure to copy the request path into a private field to make it impossible
        // to accidently swap the strings, which would cause `Path` to have an invalid reference.
        let path_buf: Box<str> = req.uri().path().into();

        // Convert this lifetime into a 'static lifetime. This allows to field to reference
        // a value in self.

        // SAFETY: We guarantee to drop this 'static reference before the owner is dropped.
        // We also never give access to this field to prevent copies from the outside.
        let path = unsafe {
            let path = path::Path::new(&path_buf);

            mem::transmute::<_, path::Path<'static>>(path)
        };

        Self {
            req,
            state,
            _path_buf: path_buf,
            path,
        }
    }

    /// Asserts that the request is authenticated and the token satisfies all [`Flags`] provided.
    /// Returns an [`enum@Error`] if this is not the case.
    pub fn require_authentication(&self, flags: Flags) -> std::result::Result<(), Error> {
        let header = self.req.authorization()?;

        let mut parts = header.split(' ');

        match parts.next() {
            Some("Bearer") => (),
            _ => return Err(StatusCodeError::bad_request().into()),
        }

        let token = match parts.next() {
            Some(token) => token,
            None => return Err(StatusCodeError::bad_request().into()),
        };

        match self.state.auth.validate_auth_token(token) {
            Ok(token) => {
                // Validates the permissions flags.
                if token.claims().flags.intersects(flags) {
                    Ok(())
                } else {
                    Err(StatusCodeError::forbidden().into())
                }
            }
            Err(_) => Err(StatusCodeError::unauthorized().into()),
        }
    }

    pub fn path(&mut self) -> &mut path::Path<'static> {
        &mut self.path
    }

    pub fn compare_etag(&self, etag: Etag) -> Option<Result> {
        // Comparing ETag and `If-Match`/`If-None-Match` headers can have two different
        // meanings. If the request method is "safe" i.e. GET or HEAD we return 304 if the
        // content DID NOT change. On all other methods we return 412 if the content DID
        // change.

        let if_match = self.req.if_match();
        let if_none_match = self.req.if_none_match();

        // We don't need to check anything if the client did not make any conditional
        // requests.
        if if_match.is_none() && if_none_match.is_none() {
            return None;
        }

        match *self.req.method() {
            // Safe methods
            Method::GET | Method::HEAD => {
                if let Some(val) = self.req.if_none_match() {
                    // TODO: This should happen the other way around, i.e. try to convert
                    // val into an Etag to avoid the allocation of `to_string`.
                    if etag.to_string().as_bytes() == val {
                        return Some(Ok(Response::not_modified()));
                    }
                }

                if let Some(val) = self.req.if_match() {
                    if etag.to_string().as_bytes() != val {
                        return Some(Err(StatusCodeError::precondition_failed().into()));
                    }
                }
            }
            // Non-safe methods
            _ => {
                if let Some(val) = self.req.if_none_match() {
                    if etag.to_string().as_bytes() == val {
                        return Some(Err(StatusCodeError::precondition_failed().into()));
                    }
                }

                if let Some(val) = self.req.if_match() {
                    if etag.to_string().as_bytes() == val {
                        return Some(Err(StatusCodeError::precondition_failed().into()));
                    }
                }
            }
        }

        None
    }
}

impl AsRef<Request> for Context {
    #[inline]
    fn as_ref(&self) -> &Request {
        &self.req
    }
}

#[derive(Debug)]
pub struct Request {
    pub parts: Parts,
    pub body: Option<Body>,
}

impl Request {
    const BODY_MAX_SIZE: usize = 16384 * 1000;

    #[inline]
    fn new(req: hyper::Request<Body>) -> Self {
        let (parts, body) = req.into_parts();

        Self {
            parts,
            body: Some(body),
        }
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

    pub fn if_match(&self) -> Option<&[u8]> {
        self.headers().get(IF_MATCH).map(|val| val.as_bytes())
    }

    pub fn if_none_match(&self) -> Option<&[u8]> {
        self.headers().get(IF_NONE_MATCH).map(|val| val.as_bytes())
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

    /// 304 Not Modified
    pub fn not_modified() -> Self {
        Self {
            status: StatusCode::NOT_MODIFIED,
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

    pub fn etag(self, etag: Etag) -> Self {
        self.header(ETAG, etag.into())
    }

    fn build(self) -> hyper::Response<Body> {
        let mut resp = hyper::Response::new(self.body);
        *resp.status_mut() = self.status;
        *resp.headers_mut() = self.headers;
        resp
    }
}

pub trait HttpResult {
    type Output;

    /// Unwraps the contained value or returns an `404 Not Found` error.
    fn map_404(self) -> std::result::Result<Self::Output, Error>;
}

impl<T> HttpResult for Option<T> {
    type Output = T;

    fn map_404(self) -> std::result::Result<T, Error> {
        match self {
            Some(val) => Ok(val),
            None => Err(StatusCodeError::not_found().into()),
        }
    }
}

impl<T, E> HttpResult for std::result::Result<Option<T>, E>
where
    E: Into<Error>,
{
    type Output = T;

    fn map_404(self) -> std::result::Result<Self::Output, Error> {
        match self {
            Ok(val) => val.map_404(),
            Err(err) => Err(err.into()),
        }
    }
}

/// Compares the etag requested in the `If-Match` and `If-None-Match` headers with the current
/// etag and returns the response if the headers matched/didn't match the current etag.
#[macro_export]
macro_rules! compare_etag {
    ($ctx:expr, $etag:expr) => {
        if let Some(res) = $ctx.compare_etag($etag) {
            return res;
        }
    };
}
