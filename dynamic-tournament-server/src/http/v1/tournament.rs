use crate::http::RequestUri;
use crate::{Error, State};

use hyper::header::HeaderValue;
use hyper::{Body, Method, Request, Response, StatusCode};

pub async fn route<'a>(
    req: Request<Body>,
    mut uri: RequestUri<'a>,
    state: State,
) -> Result<Response<Body>, Error> {
    let path = uri.take();

    match path {
        None => match req.method() {
            &Method::GET => list(req, state).await,
            &Method::POST => create(req, state).await,
            &Method::OPTIONS => Ok(Response::builder()
                .status(204)
                .body(Body::from("No Content"))
                .unwrap()),
            _ => Err(Error::MethodNotAllowed),
        },
        Some(id) => {
            let id: u64 = id.parse()?;

            match uri.take_str() {
                Some("bracket") => match req.method() {
                    &Method::GET => get_bracket(req, id, state).await,
                    &Method::PUT => put_bracket(req, id, state).await,
                    &Method::OPTIONS => Ok(Response::builder()
                        .status(204)
                        .body(Body::from("No Content"))
                        .unwrap()),

                    _ => Err(Error::MethodNotAllowed),
                },
                None => match req.method() {
                    &Method::GET => get(req, id, state).await,
                    &Method::OPTIONS => Ok(Response::builder()
                        .status(204)
                        .body(Body::from("No Content"))
                        .unwrap()),
                    _ => Err(Error::MethodNotAllowed),
                },
                _ => Err(Error::NotFound),
            }
        }
    }
}

async fn list(_req: Request<Body>, state: State) -> Result<Response<Body>, Error> {
    let ids = state.list_tournaments().await?;

    let body = serde_json::to_string(&ids)?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();

    Ok(resp)
}

async fn create(req: Request<Body>, state: State) -> Result<Response<Body>, Error> {
    if !state.is_authenticated(&req) {
        return Ok(Response::builder()
            .status(403)
            .body(Body::from("Forbidden"))
            .unwrap());
    }

    let bytes = hyper::body::to_bytes(req.into_body()).await?;

    let mut resp = Response::new(Body::empty());

    let tournament = match serde_json::from_slice(&bytes) {
        Ok(tournament) => tournament,
        Err(err) => {
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            *resp.body_mut() = Body::from(err.to_string());

            return Ok(resp);
        }
    };

    let id = state.create_tournament(tournament).await?;

    *resp.status_mut() = StatusCode::CREATED;
    *resp.body_mut() = Body::from(id.to_string());

    Ok(resp)
}

async fn get(_req: Request<Body>, id: u64, state: State) -> Result<Response<Body>, Error> {
    let mut resp = Response::new(Body::empty());

    resp.headers_mut()
        .append("Content-Type", HeaderValue::from_static("application/json"));

    match state.get_tournament(id).await? {
        Some(t) => {
            *resp.status_mut() = StatusCode::OK;
            *resp.body_mut() = Body::from(serde_json::to_string(&t).unwrap());
        }
        None => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Ok(resp)
}

async fn put_bracket(req: Request<Body>, id: u64, state: State) -> Result<Response<Body>, Error> {
    if !state.is_authenticated(&req) {
        return Ok(Response::builder()
            .status(403)
            .body(Body::from("Forbidden"))
            .unwrap());
    }

    let bytes = hyper::body::to_bytes(req.into_body()).await?;

    let mut resp = Response::new(Body::empty());

    let bracket = match serde_json::from_slice(&bytes) {
        Ok(bracket) => bracket,
        Err(err) => {
            *resp.status_mut() = StatusCode::BAD_GATEWAY;
            *resp.body_mut() = Body::from(err.to_string());

            return Ok(resp);
        }
    };

    state.update_bracket(id, bracket).await?;

    *resp.status_mut() = StatusCode::OK;

    Ok(resp)
}

async fn get_bracket(_req: Request<Body>, id: u64, state: State) -> Result<Response<Body>, Error> {
    match state.get_tournament(id).await? {
        Some(_) => (),
        None => {
            let resp = Response::builder()
                .status(404)
                .body(Body::from("Not found"))
                .unwrap();

            return Ok(resp);
        }
    };

    let bracket = match state.get_bracket(id).await? {
        Some(b) => b,
        None => {
            let resp = Response::builder()
                .status(404)
                .body(Body::from("Not Found"))
                .unwrap();

            return Ok(resp);
        }
    };

    let resp = Response::builder()
        .status(200)
        .header("Content-Type", "application/json");

    let body = serde_json::to_string(&bracket)?;

    Ok(resp.body(Body::from(body)).unwrap())
}
