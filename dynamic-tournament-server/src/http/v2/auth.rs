use crate::http::{Context, Response, Result};
use crate::StatusCodeError;

use dynamic_tournament_api::auth::Claims;
use dynamic_tournament_api::v3::auth::RefreshToken;
use dynamic_tournament_macros::{method, path};
use hyper::header::{HeaderValue, AUTHORIZATION, COOKIE};

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

async fn login(ctx: Context) -> Result {
    wp_validate(&ctx).await?;

    let tokens = ctx.state.auth.create_tokens(Claims::new(0))?;
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

/// We validate whether a request is as follows:
/// 1. Get the authorization and cookie headers from the incoming request. If both are missing
/// return 401.
/// 2. Make a request to the wordpress backend with the authorization and cookie headers attached.
/// 3. If the request fails for any reason we reject with 500.
/// 4. If the returned status of the request to the wordpress API is 200 and only 200, we return
/// accept the request. If the status code is not 200, it should be a 401 if correctly configured,
/// we also return 401.
async fn wp_validate(ctx: &Context) -> Result {
    let uri = format!("{}/api/wp/v2/users", ctx.state.config.wordpress.upstream);
    log::debug!("Validating using upstream {}", uri);

    let mut req = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::parse(&uri).unwrap());
    req.headers_mut().append(
        "Host",
        HeaderValue::from_str(&ctx.state.config.wordpress.host).unwrap(),
    );

    if let Some(val) = ctx.req.headers().get(AUTHORIZATION) {
        req.headers_mut().append(AUTHORIZATION, val.clone());
    }

    if let Some(val) = ctx.req.headers().get(COOKIE) {
        req.headers_mut().append(COOKIE, val.clone());
    }

    if req.headers().get(AUTHORIZATION).is_none() && req.headers().get(COOKIE).is_none() {
        return Err(StatusCodeError::unauthorized().into());
    }

    let client = reqwest::Client::new();

    let resp = match client.execute(req).await {
        Ok(resp) => resp,
        Err(err) => {
            log::error!("Failed to fetch wordpress upstream: {}", err);
            return Err(StatusCodeError::internal_server_error().into());
        }
    };

    // Validate that the server returned an success status code.
    if resp.status() != 200 {
        log::debug!(
            "Request is not valid: Wordpress upstream returned {}",
            resp.status()
        );

        return Err(StatusCodeError::unauthorized().into());
    }

    Ok(Response::ok())
}
