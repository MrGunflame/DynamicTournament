mod systems;
mod tournaments;

use crate::http::{Request, RequestUri, Result};
use crate::StatusCodeError;

use super::v2;

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    match uri.take_str() {
        // /v3/auth uses the same endpoint as /v2/auth.
        Some("auth") => v2::auth::route(req, uri).await,
        Some("tournaments") => tournaments::route(req, uri).await,
        Some("systems") => systems::route(req, uri).await,
        _ => Err(StatusCodeError::not_found().into()),
    }
}
