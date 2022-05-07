use crate::http::{Request, RequestUri};
use crate::{Error, State, StatusCodeError};

use hyper::header::{HeaderValue, CONNECTION, UPGRADE};
use hyper::{Body, Method, Response, StatusCode};
use sha1::{Digest, Sha1};

pub async fn route<'a>(
    req: Request,
    mut uri: RequestUri<'a>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take() {
        None => match *req.method() {
            Method::GET => list(req, state).await,
            Method::POST => create(req, state).await,
            Method::OPTIONS => Ok(Response::builder()
                .status(204)
                .body(Body::from("No Content"))
                .unwrap()),
            _ => Err(StatusCodeError::method_not_allowed().into()),
        },
        Some(id) => {
            let id: u64 = id.parse()?;

            match uri.take_str() {
                Some("bracket") => match *req.method() {
                    Method::GET => bracket(req, id, state).await,
                    Method::OPTIONS => Ok(Response::builder()
                        .status(204)
                        .body(Body::from("No Content"))
                        .unwrap()),
                    _ => Err(StatusCodeError::method_not_allowed().into()),
                },
                None => match *req.method() {
                    Method::GET => get(req, id, state).await,
                    Method::OPTIONS => Ok(Response::builder()
                        .status(204)
                        .body(Body::from("No Content"))
                        .unwrap()),
                    _ => Err(StatusCodeError::method_not_allowed().into()),
                },
                _ => Err(StatusCodeError::not_found().into()),
            }
        }
    }
}

async fn list(_req: Request, state: State) -> Result<Response<Body>, Error> {
    let ids = state.list_tournaments().await?;

    let body = serde_json::to_string(&ids)?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();

    Ok(resp)
}

async fn create(req: Request, state: State) -> Result<Response<Body>, Error> {
    if !state.is_authenticated(&req) {
        return Ok(Response::builder()
            .status(403)
            .body(Body::from("Forbidden"))
            .unwrap());
    }

    let tournament = req.json().await?;

    let mut resp = Response::new(Body::empty());

    let id = state.create_tournament(tournament).await?;

    *resp.status_mut() = StatusCode::CREATED;
    *resp.body_mut() = Body::from(id.to_string());

    Ok(resp)
}

async fn get(_req: Request, id: u64, state: State) -> Result<Response<Body>, Error> {
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

pub async fn bracket(mut req: Request, id: u64, state: State) -> Result<Response<Body>, Error> {
    let mut resp = Response::new(Body::empty());

    if !req.headers().contains_key(UPGRADE) {
        match state.get_bracket(id).await? {
            Some(bracket) => {
                *resp.status_mut() = StatusCode::OK;
                resp.headers_mut()
                    .insert("Content-Type", HeaderValue::from_static("application/json"));

                let body = serde_json::to_string(&bracket)?;
                *resp.body_mut() = Body::from(body);
            }
            None => {
                *resp.status_mut() = StatusCode::NOT_FOUND;
                *resp.body_mut() = Body::from("Not Found");
            }
        }

        return Ok(resp);
    }

    log::info!("Upgraded connection");

    if let Some(value) = req.headers().get("Sec-WebSocket-Key") {
        let value = value.to_str().unwrap().to_owned() + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

        let mut hasher = Sha1::new();
        hasher.update(value.as_bytes());
        let result = hasher.finalize();

        let val = base64::encode(result);

        resp.headers_mut()
            .insert("Sec-WebSocket-Accept", HeaderValue::from_str(&val).unwrap());
    }

    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut req.request).await {
            Ok(conn) => crate::websocket::handle(conn, state, id).await,
            Err(err) => log::error!("Failed to upgrade connection: {:?}", err),
        }
    });

    resp.headers_mut()
        .insert("Sec-WebSocket-Version", HeaderValue::from_static("13"));

    *resp.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    resp.headers_mut()
        .insert(CONNECTION, HeaderValue::from_static("Upgrade"));
    resp.headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("websocket"));
    Ok(resp)
}
