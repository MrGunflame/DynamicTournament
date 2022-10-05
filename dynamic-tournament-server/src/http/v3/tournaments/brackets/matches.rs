use crate::http::{Context, Response, Result};
use crate::StatusCodeError;

use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_macros::{method, path};
use hyper::header::{
    HeaderValue, CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, SEC_WEBSOCKET_VERSION,
    UPGRADE,
};
use sha1::{Digest, Sha1};

use tokio::task;

pub async fn route(mut ctx: Context, id: TournamentId, bracket_id: BracketId) -> Result {
    path!(ctx, {
        @ => method!(ctx, {
            GET => serve(ctx, id, bracket_id).await,
        })
    })
}

async fn serve(ctx: Context, id: TournamentId, bracket_id: BracketId) -> Result {
    // Check that the tournament and bracket exist.
    if ctx.state.store.get_tournament(id).await?.is_none()
        || ctx.state.store.get_bracket(id, bracket_id).await?.is_none()
    {
        return Err(StatusCodeError::not_found().into());
    }

    if !ctx.req.headers().contains_key(UPGRADE) {
        return Err(StatusCodeError::upgrade_required().into());
    }

    let mut resp = Response::switching_protocols();

    // Set the `Sec-WebSocket-Accept` header when the request contains the `Sec-WebSocket-Key`
    // header.
    if let Some(value) = ctx.req.headers().get(SEC_WEBSOCKET_KEY) {
        let mut value = value.as_bytes().to_vec();
        value.extend(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

        let mut hasher = Sha1::new();
        hasher.update(value);

        let res = base64::encode(hasher.finalize());
        let header = HeaderValue::from_str(&res).unwrap();

        resp = resp.header(SEC_WEBSOCKET_ACCEPT, header)
    }

    log::debug!("Upgrading connection");

    task::spawn(async move {
        let state = ctx.state.clone();
        let req = hyper::Request::from_parts(ctx.req.parts, ctx.req.body.unwrap());

        match hyper::upgrade::on(req).await {
            Ok(conn) => crate::websocket::handle(conn, state, id, bracket_id).await,
            Err(err) => log::error!("Failed to upgrade connection: {:?}", err),
        }
    });

    resp = resp
        .header(CONNECTION, HeaderValue::from_static("Upgrade"))
        .header(UPGRADE, HeaderValue::from_static("websocket"))
        .header(SEC_WEBSOCKET_VERSION, HeaderValue::from_static("13"));

    log::debug!("Upgraded connection");

    Ok(resp)
}
