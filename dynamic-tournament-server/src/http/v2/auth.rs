use crate::auth::password_hash;
use crate::http::{Request, RequestUri, Response, Result};
use crate::StatusCodeError;

use dynamic_tournament_api::auth::Claims;
use dynamic_tournament_api::v3::auth::RefreshToken;
use dynamic_tournament_api::v3::users::User;
use dynamic_tournament_macros::method;

pub async fn route(req: Request, uri: RequestUri<'_>) -> Result {
    match uri.take_all() {
        Some("login") => method!(req, {
            POST => login(req).await,
        }),
        Some("refresh") => method!(req, {
            POST => refresh(req).await,
        }),
        _ => Err(StatusCodeError::not_found().into()),
    }
}

async fn login(mut req: Request) -> Result {
    let input: User = req.json().await?;

    // Find a user with matching username in the database. Return 401 when the
    // username doesn't exist.
    let user = match req.state().store.users().get(&input.username).await? {
        Some(user) => user,
        None => return Err(StatusCodeError::unauthorized().into()),
    };

    // Get the salted password hash by first hashing the password, then the user id.
    // Note that the id is passed as bytes. The id is not converted to a string.
    let hash = password_hash(input.password, user.id.0.to_le_bytes());

    // Match password hashes
    if hash != user.password {
        return Err(StatusCodeError::unauthorized().into());
    }

    let tokens = req.state().auth.create_tokens(Claims::new(user.id.0))?;
    Ok(Response::ok().json(&tokens))
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
