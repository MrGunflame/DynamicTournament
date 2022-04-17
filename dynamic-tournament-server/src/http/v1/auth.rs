use hyper::Method;
use hyper::{Body, Request, Response};

use crate::{http::RequestUri, Error, State};

pub async fn route<'a>(
    req: Request<Body>,
    mut uri: RequestUri<'a>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take_str() {
        Some("login") => match req.method() {
            &Method::POST => login(req, state).await,
            &Method::OPTIONS => Ok(Response::builder()
                .status(204)
                .body(Body::from("No Content"))
                .unwrap()),

            _ => Err(Error::MethodNotAllowed),
        },
        _ => Err(Error::NotFound),
    }
}

async fn login(req: Request<Body>, state: State) -> Result<Response<Body>, Error> {
    let bytes = hyper::body::to_bytes(req.into_body()).await?;

    let resp = Response::builder();

    let data = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(err) => {
            return Ok(resp.status(400).body(Body::from(err.to_string())).unwrap());
        }
    };

    Ok(if state.is_allowed(&data) {
        resp.status(200).body(Body::from("OK")).unwrap()
    } else {
        resp.status(403).body(Body::from("Forbidden")).unwrap()
    })
}
