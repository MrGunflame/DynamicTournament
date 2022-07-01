mod systems;
mod tournaments;

use hyper::{Body, Response};

use crate::http::{Request, RequestUri};
use crate::{Error, StatusCodeError};

use super::v2;

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result<Response<Body>, Error> {
    match uri.take_str() {
        // /v3/auth uses the same endpoint as /v2/auth.
        Some("auth") => v2::auth::route(req, uri).await,
        Some("tournaments") => tournaments::route(req, uri).await,
        Some("systems") => systems::route(req, uri).await,
        _ => Err(StatusCodeError::not_found().into()),
    }
}
