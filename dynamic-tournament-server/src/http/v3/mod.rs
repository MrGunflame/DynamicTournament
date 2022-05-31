mod systems;
mod tournaments;

use hyper::{Body, Response};

use crate::http::{Request, RequestUri};
use crate::{Error, State, StatusCodeError};

pub async fn route(
    req: Request,
    mut uri: RequestUri<'_>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take_str() {
        Some("tournaments") => tournaments::route(req, uri, state).await,
        Some("systems") => systems::route(req, uri, state).await,
        _ => Err(StatusCodeError::not_found().into()),
    }
}
