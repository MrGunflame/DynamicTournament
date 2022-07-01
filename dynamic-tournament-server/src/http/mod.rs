mod v1;
pub mod v2;
mod v3;

use crate::{Error, State, StatusCodeError};

use std::convert::Infallible;
use std::net::SocketAddr;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};
use std::time::Duration;

use futures::future::BoxFuture;
use futures::Future;
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::server::conn::Http;
use hyper::service::Service;
use hyper::{Body, HeaderMap, Method, Response, StatusCode, Uri};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::net::TcpSocket;
use tokio::time::Instant;

pub async fn bind(addr: SocketAddr, state: State) -> Result<(), crate::Error> {
    let mut shutdown_rx = state.shutdown_rx.clone();

    let service = RootService { state };

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

                let service = service.clone();
                let mut shutdown_rx = shutdown_rx.clone();
                tokio::task::spawn(async move {
                    let mut conn = Http::new()
                        .http1_keep_alive(true)
                        .serve_connection(stream, service)
                        .with_upgrades();

                    let mut conn = Pin::new(&mut conn);

                    tokio::select! {
                        res = &mut conn => {
                            if let Err(err) = res {
                                log::warn!("Http error: {:?}", err);
                            }
                        }
                        _ = shutdown_rx.changed() => {
                            log::debug!("Shutting down connection");
                            conn.graceful_shutdown();
                        }
                    }
                });
            }
            // Shut down the server.
            _ = shutdown_rx.changed() => {
                log::debug!("Shutting down http server");
                return Ok(());
            }
        }
    }
}

#[derive(Clone, Debug)]
struct RootService {
    state: State,
}

impl Service<hyper::Request<Body>> for RootService {
    type Response = Response<Body>;
    type Error = crate::Error;
    type Future = RootServiceFuture;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    #[inline]
    fn call(&mut self, req: hyper::Request<Body>) -> Self::Future {
        RootServiceFuture::new(req, self.state.clone())
    }
}

struct RootServiceFuture(BoxFuture<'static, Result<Response<Body>, crate::Error>>);

impl RootServiceFuture {
    fn new(req: hyper::Request<Body>, state: State) -> Self {
        Self(Box::pin(async move {
            Ok(service_root(req, state).await.unwrap())
        }))
    }
}

impl Future for RootServiceFuture {
    type Output = Result<Response<Body>, crate::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let future = unsafe { self.map_unchecked_mut(|this| &mut this.0) };
        future.poll(cx)
    }
}

async fn service_root(
    req: hyper::Request<Body>,
    state: State,
) -> Result<Response<Body>, Infallible> {
    log::trace!("Received Request:");
    log::trace!("Head: {} {}", req.method(), req.uri());
    log::trace!("Headers: {:?}", req.headers());
    log::trace!("Body: {:?}", req.body());

    let req = Request { request: req };

    if req.method() == Method::POST {
        let mut resp = Response::new(Body::empty());
        match req.headers().get("Content-Length") {
            Some(value) => match value.to_str() {
                Ok(s) => match s.parse::<u64>() {
                    Ok(length) => {
                        if length > 16384 {
                            *resp.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                            *resp.body_mut() = Body::from("Payload Too Large");
                            return Ok(resp);
                        }
                    }
                    // Content-Length header is malformed.
                    _ => {
                        *resp.status_mut() = StatusCode::BAD_REQUEST;
                        *resp.body_mut() = Body::from("Bad Request");
                        return Ok(resp);
                    }
                },
                // Content-Length header is malformed.
                _ => {
                    *resp.status_mut() = StatusCode::BAD_REQUEST;
                    *resp.body_mut() = Body::from("Bad Request");
                    return Ok(resp);
                }
            },
            // Content-Length header is missing.
            None => {
                *resp.status_mut() = StatusCode::LENGTH_REQUIRED;
                *resp.body_mut() = Body::from("Length Required");
                return Ok(resp);
            }
        }
    }

    let uri = String::from(req.uri().path());

    let mut uri = RequestUri::new(&uri);

    log::debug!("{:?}", uri);

    let origin = req.headers().get("Origin").cloned();

    let res = match uri.take_str() {
        Some("v1") => v1::route().await,
        Some("v2") => v2::route(req, uri, state).await,
        Some("v3") => v3::route(req, uri, state).await,
        _ => Err(Error::NotFound),
    };

    match res {
        Ok(mut resp) => {
            log::debug!("Settings CORS for origin: {:?}", origin);
            if let Some(origin) = origin {
                resp.headers_mut()
                    .append("Access-Control-Allow-Origin", origin);
            }

            for (k, v) in [(
                "Access-Control-Allow-Headers",
                "content-type, authorization",
            )] {
                resp.headers_mut().append(k, HeaderValue::from_static(v));
            }

            Ok(resp)
        }
        Err(err) => {
            let mut resp = Response::new(Body::empty());

            match err {
                Error::NotFound => {
                    *resp.status_mut() = StatusCode::NOT_FOUND;
                    *resp.body_mut() = Body::from("Not Found");
                }
                Error::BadRequest => {
                    *resp.status_mut() = StatusCode::BAD_REQUEST;
                    *resp.body_mut() = Body::from("Bad Request");
                }
                Error::MethodNotAllowed => {
                    *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
                    *resp.body_mut() = Body::from("Method Not Allowed");
                }
                Error::StatusCodeError(err) => {
                    let body = serde_json::to_vec(&ErrorResponse {
                        code: err.code.as_u16(),
                        message: err.message,
                    })
                    .unwrap();

                    *resp.status_mut() = err.code;
                    resp.headers_mut()
                        .append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                    *resp.body_mut() = Body::from(body);
                }
                err => {
                    log::error!("{:?}", err);

                    *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    *resp.body_mut() = Body::from("Internal Server Error");
                }
            }

            Ok(resp)
        }
    }
}

#[derive(Debug)]
pub struct Request {
    pub request: hyper::Request<Body>,
}

impl Request {
    pub fn method(&self) -> &Method {
        self.request.method()
    }

    pub fn headers(&self) -> &HeaderMap<HeaderValue> {
        self.request.headers()
    }

    pub fn uri(&self) -> &Uri {
        self.request.uri()
    }

    pub async fn json<T>(self) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        const DUR: Duration = Duration::new(30, 0);

        let deadline = Instant::now() + DUR;

        let bytes = tokio::select! {
            res = hyper::body::to_bytes(self.request.into_body()) => {
                res?
            }
            _ = tokio::time::sleep_until(deadline) => {
                log::info!("Client failed to transmit body in {}s, dropping connection", DUR.as_secs());
                return Err(StatusCodeError::request_timeout().into());
            }
        };

        match serde_json::from_slice(&bytes) {
            Ok(value) => Ok(value),
            Err(err) => Err(StatusCodeError::new(StatusCode::BAD_REQUEST, err).into()),
        }
    }

    /// Returns the value of the "Content-Length" header. If the header is not present or has an
    /// invalid value an error is returned.
    pub fn content_length(&self) -> Result<u64, Error> {
        match self.request.headers().get("Content-Length") {
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
    pub fn parse<T>(&self) -> Result<T, Error>
    where
        T: FromStr,
    {
        match self.part.parse() {
            Ok(v) => Ok(v),
            Err(_) => Err(Error::BadRequest),
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
                use hyper::{Response, Body};
                use hyper::header::{HeaderValue, ALLOW, ACCESS_CONTROL_ALLOW_METHODS};

                let allow = vec![$($method.as_str()),*];
                let allow = HeaderValue::from_bytes(allow.join(",").as_bytes()).unwrap();

                let mut resp = Response::new(Body::empty());
                *resp.status_mut() = hyper::StatusCode::NO_CONTENT;

                let headers = &mut *resp.headers_mut();
                headers.insert(ALLOW, allow.clone());
                headers.insert(ACCESS_CONTROL_ALLOW_METHODS, allow);

                Ok(resp)
            }
            _ => Err(crate::StatusCodeError::method_not_allowed().into()),
        }
    };
}
