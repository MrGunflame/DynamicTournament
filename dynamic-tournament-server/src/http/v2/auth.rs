use chrono::Utc;
use hyper::Method;
use jsonwebtoken::{EncodingKey, Header};

use crate::http::{Request, RequestUri, Response, Result};
use crate::{method, StatusCodeError};

use dynamic_tournament_api::auth::{Claims, Token, TokenPair};
use dynamic_tournament_api::v3::auth::RefreshToken;

/// Auth token expiration time. Defaults to 1 hour.
const AUTH_TOKEN_EXP: u64 = 60 * 60;
/// Refresh token expiration time. Defaults to 24 hours.
const REFRESH_TOKEN_EXPR: u64 = 60 * 60 * 24;

pub const SECRET: &[u8] = include_bytes!("../../../jwt-secret");

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
        let tokens = create_token_pair(Claims::default())?;

        Ok(Response::ok().json(&tokens))
    } else {
        Err(StatusCodeError::unauthorized().into())
    }
}

async fn refresh(mut req: Request) -> Result {
    let body: RefreshToken = req.json().await?;

    let claims = match req.state().decode_token(body.refresh_token.token()) {
        Ok(claims) => claims,
        Err(err) => {
            log::info!("Failed to decode jwt token: {:?}", err);

            return Err(StatusCodeError::unauthorized().into());
        }
    };

    let body = create_token_pair(claims)?;

    Ok(Response::ok().json(&body))
}

fn create_token_pair(
    mut claims: Claims,
) -> std::result::Result<TokenPair, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as u64;

    claims.iat = now;
    claims.nbf = now;
    claims.exp = now + AUTH_TOKEN_EXP;

    let header = Header::default();
    let key = EncodingKey::from_secret(SECRET);

    let auth_token = jsonwebtoken::encode(&header, &claims, &key)?;
    let auth_token = Token::from_parts(auth_token, claims.clone());

    claims.exp = now + REFRESH_TOKEN_EXPR;

    let refresh_token = jsonwebtoken::encode(&header, &claims, &key)?;
    let refresh_token = Token::from_parts(refresh_token, claims);

    Ok(TokenPair {
        auth_token,
        refresh_token,
    })
}
