use crate::http::{Request, RequestUri};
use crate::method;
use crate::{Error, State, StatusCodeError};

use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use hyper::header::{
    HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION,
    UPGRADE,
};
use hyper::{Body, Method, Response, StatusCode};
use sha1::{Digest, Sha1};

use tokio::task;

pub async fn route(
    req: Request,
    mut uri: RequestUri<'_>,
    state: State,
    id: TournamentId,
    bracket_id: BracketId,
) -> Result<Response<Body>, Error> {
    if uri.take().is_none() {
        method!(req, {
            Method::GET => serve(req, id, bracket_id, state).await,
        })
    } else {
        Err(StatusCodeError::not_found().into())
    }
}

async fn serve(
    mut req: Request,
    id: TournamentId,
    bracket_id: BracketId,
    state: State,
) -> Result<Response<Body>, Error> {
    // Check that the tournament and bracket exist.
    if state.store.get_tournament(id).await?.is_none()
        || state.store.get_bracket(id, bracket_id).await?.is_none()
    {
        return Err(StatusCodeError::not_found().into());
    }

    if !req.headers().contains_key(UPGRADE) {
        return Err(StatusCodeError::upgrade_required().into());
    }

    let mut resp = Response::new(Body::empty());

    // Set the `Sec-WebSocket-Accept` header when the request contains the `Sec-WebSocket-Key`
    // header.
    if let Some(value) = req.headers().get(SEC_WEBSOCKET_KEY) {
        let mut value = value.as_bytes().to_vec();
        value.extend(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

        let mut hasher = Sha1::new();
        hasher.update(value);

        let res = base64::encode(hasher.finalize());
        let header = HeaderValue::from_str(&res).unwrap();

        resp.headers_mut().insert(SEC_WEBSOCKET_ACCEPT, header);
    }

    log::debug!("Upgrading connection");

    task::spawn(async move {
        match hyper::upgrade::on(&mut req.request).await {
            Ok(conn) => crate::websocket::handle(conn, state, id, bracket_id).await,
            Err(err) => log::error!("Failed to upgrade connection: {:?}", err),
        }
    });

    *resp.status_mut() = StatusCode::SWITCHING_PROTOCOLS;

    let headers = resp.headers_mut();
    headers.insert(CONNECTION, HeaderValue::from_static("Upgrade"));
    headers.insert(UPGRADE, HeaderValue::from_static("websocket"));
    headers.insert(SEC_WEBSOCKET_VERSION, HeaderValue::from_static("13"));

    log::debug!("Upgraded connection");

    Ok(resp)
}
