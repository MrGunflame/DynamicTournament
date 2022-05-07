pub mod auth;
mod tournament;

use crate::http::{Request, RequestUri};
use crate::{Error, State, StatusCodeError};

use hyper::{Body, Response};

pub async fn route<'a>(
    req: Request,
    mut uri: RequestUri<'a>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take_str() {
        Some("tournament") => tournament::route(req, uri, state).await,
        Some("auth") => auth::route(req, uri, state).await,
        _ => Err(StatusCodeError::not_found().into()),
    }
}
