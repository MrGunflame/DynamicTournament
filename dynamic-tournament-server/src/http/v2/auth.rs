use hyper::Method;

use crate::http::{Request, RequestUri, Response, Result};
use crate::{method, StatusCodeError};

use dynamic_tournament_api::auth::Claims;
use dynamic_tournament_api::v3::auth::RefreshToken;

pub async fn route(req: Request, uri: RequestUri<'_>) -> Result {
    match uri.take_all() {
        Some("login") => method!(req, {
            Method::POST => login(req).await,
        }),
        Some("refresh") => method!(req, {
            Method::POST => refresh(req).await,
        }),
        _ => Err(StatusCodeError::not_found().into()),
    }
}

async fn login(mut req: Request) -> Result {
    let data = req.json().await?;

    if req.state().is_allowed(&data) {
        let tokens = req.state().auth.create_tokens(Claims::default())?;
        Ok(Response::ok().json(&tokens))
    } else {
        Err(StatusCodeError::unauthorized().into())
    }
}

async fn refresh(mut req: Request) -> Result {
    let body: RefreshToken = req.json().await?;

    match req.state().auth.validate_refresh_token(body.refresh_token) {
        Ok(token) => {
            let tokens = req.state().auth.create_tokens(token.into_claims())?;
            Ok(Response::ok().json(&tokens))
        }
        Err(_) => Err(StatusCodeError::unauthorized().into()),
    }
}
