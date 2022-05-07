mod v1;
pub mod v2;

use crate::{Error, State, StatusCodeError};

use std::convert::Infallible;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use hyper::header::HeaderValue;
use hyper::server::Server;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, HeaderMap, Method, Response, StatusCode, Uri};
use serde::de::DeserializeOwned;
use tokio::time::Instant;

pub async fn bind(addr: SocketAddr, state: State) -> Result<(), hyper::Error> {
    let make_svc = make_service_fn(move |_conn| {
        let state = state.clone();
        async move {
            Ok::<_, Infallible>(service_fn({
                move |req| {
                    let state = state.clone();
                    service_root(req, state)
                }
            }))
        }
    });

    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(shutdown_signal());

    server.await
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c().await.unwrap()
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
        _ => Err(Error::NotFound),
    };

    match res {
        Ok(mut resp) => {
            log::debug!("Settings CORS for origin: {:?}", origin);
            if let Some(origin) = origin {
                resp.headers_mut()
                    .append("Access-Control-Allow-Origin", origin);
            }

            for (k, v) in [
                ("Access-Control-Allow-Methods", "GET, POST, OPTIONS, PUT"),
                (
                    "Access-Control-Allow-Headers",
                    "content-type, authorization",
                ),
            ] {
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
                    *resp.status_mut() = err.code;
                    *resp.body_mut() = Body::from(err.message);
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
        if path.starts_with("/") {
            path = &path[1..];
        }

        Self { path }
    }

    pub fn take(&mut self) -> Option<UriPart> {
        let part = self.take_str()?;

        let part = UriPart { part: part };

        Some(part)
    }

    pub fn take_str(&mut self) -> Option<&str> {
        if self.path.is_empty() {
            None
        } else {
            Some(match self.path.split_once("/") {
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
        &self.part
    }
}
