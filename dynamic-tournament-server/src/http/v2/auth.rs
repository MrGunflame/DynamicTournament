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

async fn wp_validate(ctx: &Context) -> Result {
    let uri = format!("{}/api/wp/v2/users", ctx.state.config.wp_upstream);
    log::debug!("Validating using upstream {}", uri);

    let mut req = reqwest::Request::new(reqwest::Method::GET, reqwest::Url::parse(&uri).unwrap());
    req.headers_mut()
        .append("Host", HeaderValue::from_static("beta.hardstuck.local"));

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
