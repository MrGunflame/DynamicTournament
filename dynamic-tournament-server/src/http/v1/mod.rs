mod auth;
mod tournament;

use crate::http::RequestUri;
use crate::{Error, State};

use hyper::{Body, Request, Response};

pub async fn route<'a>(
    req: Request<Body>,
    mut uri: RequestUri<'a>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take_str() {
        Some("tournament") => tournament::route(req, uri, state).await,
        Some("auth") => auth::route(req, uri, state).await,
        _ => Err(Error::NotFound),
    }
}
