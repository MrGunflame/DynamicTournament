use crate::auth::password_hash;
use crate::http::{Context, Response, Result};
use crate::StatusCodeError;

use dynamic_tournament_api::auth::{Claims, Flags};
use dynamic_tournament_api::v3::auth::RefreshToken;
use dynamic_tournament_api::v3::users::User;
use dynamic_tournament_macros::{method, path};

pub async fn route(mut ctx: Context) -> Result {
    path!(ctx, {
        "login" => method!(ctx, {
            POST => login(ctx).await,
        }),
        "refresh" => method!(ctx, {
            POST => refresh(ctx).await,
        }),
    })
}

async fn login(mut ctx: Context) -> Result {
    let input: User = ctx.req.json().await?;

    // Find a user with matching username in the database. Return 401 when the
    // username doesn't exist.
    let user = match ctx.state.store.users().get(&input.username).await? {
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

    // Set all permissions
    let mut claims = Claims::new(user.id.0);
    claims.flags = Flags::ALL;

    let tokens = ctx.state.auth.create_tokens(claims)?;
    Ok(Response::ok().json(&tokens))
}

async fn refresh(mut ctx: Context) -> Result {
    let body: RefreshToken = ctx.req.json().await?;

    match ctx.state.auth.validate_refresh_token(body.refresh_token) {
        Ok(token) => {
            let tokens = ctx.state.auth.create_tokens(token.into_claims())?;
            Ok(Response::ok().json(&tokens))
        }
        Err(_) => Err(StatusCodeError::unauthorized().into()),
    }
}
