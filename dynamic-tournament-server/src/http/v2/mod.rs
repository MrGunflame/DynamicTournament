pub mod auth;
mod tournament;

use crate::http::{Request, RequestUri, Result};
use crate::StatusCodeError;

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    match uri.take_str() {
        Some("tournament") => tournament::route(req, uri).await,
        Some("auth") => auth::route(req, uri).await,
        _ => Err(StatusCodeError::not_found().into()),
    }
}
