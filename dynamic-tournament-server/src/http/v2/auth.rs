use std::borrow::Cow;
use std::collections::HashMap;

use crate::http::{Context, Response, Result};
use crate::StatusCodeError;

use dynamic_tournament_api::auth::{Claims, Flags};
use dynamic_tournament_api::v3::auth::RefreshToken;
use dynamic_tournament_macros::{method, path};
use hyper::header::{AUTHORIZATION, COOKIE, HOST};
use serde::{Deserialize, Serialize};

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
    wp_validate(&ctx).await
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
    let uri = format!(
        "{}/api/wp/v2/users/me?context=edit",
        ctx.state.config.wordpress.upstream
    );
    log::debug!("Validating using upstream {}", uri);

    let uri = match reqwest::Url::parse(&uri) {
        Ok(uri) => uri,
        Err(err) => {
            log::error!(
                "Failed to create uri: {} (using {:?}), is the upstream configured correctly?",
                err,
                uri
            );
            return Err(StatusCodeError::internal_server_error().into());
        }
    };

    let mut req = reqwest::Request::new(reqwest::Method::GET, uri);

    // Append the host header as received.
    if let Some(val) = ctx.req.headers().get(HOST) {
        req.headers_mut().append(HOST, val.clone());
    }

    if let Some(val) = ctx.req.headers().get(AUTHORIZATION) {
        req.headers_mut().append(AUTHORIZATION, val.clone());
    }

    if let Some(val) = ctx.req.headers().get(COOKIE) {
        req.headers_mut().append(COOKIE, val.clone());
    }

    // The `X-WP-Nonce` header is required for `Cookie` requests.
    if let Some(val) = ctx.req.headers().get("X-WP-Nonce") {
        req.headers_mut().append("X-WP-Nonce", val.clone());
    }

    // Either a `Authorization` or `Cookie` header is required.
    if req.headers().get(AUTHORIZATION).is_none() && req.headers().get(COOKIE).is_none() {
        log::debug!("No Cookie or Authorization header, rejecting");
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

    let body = match resp.bytes().await {
        Ok(body) => body,
        Err(err) => {
            log::debug!("Failed to stream body: {}", err);
            return Err(StatusCodeError::internal_server_error().into());
        }
    };

    let user = match serde_json::from_slice::<WpUser>(&body) {
        Ok(user) => user,
        Err(err) => {
            log::debug!("Failed to deserialize response body: {}", err);
            return Err(StatusCodeError::internal_server_error().into());
        }
    };

    let tokens = ctx.state.auth.create_tokens(user.claims())?;

    log::debug!(
        "Issuing token with sub={:?} and flags={:?}",
        tokens.auth_token.claims().sub,
        tokens.auth_token.claims().flags,
    );

    Ok(Response::ok().json(&tokens))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct WpUser<'a> {
    id: u64,
    username: Cow<'a, str>,
    name: Cow<'a, str>,
    capabilities: HashMap<Cow<'a, str>, bool>,
}

impl<'a> WpUser<'a> {
    fn claims(&self) -> Claims {
        let mut claims = Claims::new(self.id);

        if self
            .capabilities
            .get("dt_tournaments_edit_scores")
            .is_some()
        {
            claims.flags |= Flags::EDIT_SCORES;
        }

        if self.capabilities.get("dt_tournaments_admin").is_some() {
            claims.flags |= Flags::ADMIN;
        }

        claims
    }
}
