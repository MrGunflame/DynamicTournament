use chrono::Utc;
use hyper::{Body, Response};
use hyper::{Method, StatusCode};
use jsonwebtoken::{EncodingKey, Header};

use crate::{
    http::{Request, RequestUri},
    Error, State,
};

use dynamic_tournament_api::auth::{Claims, RefreshToken, TokenPair};

/// Auth token expiration time. Defaults to 1 hour.
const AUTH_TOKEN_EXP: u64 = 60 * 60;
/// Refresh token expiration time. Defaults to 24 hours.
const REFRESH_TOKEN_EXPR: u64 = 60 * 60 * 24;

pub const SECRET: &'static [u8] = include_bytes!("../../../jwt-secret");

pub async fn route<'a>(
    req: Request,
    uri: RequestUri<'a>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take_all() {
        Some("login") => match *req.method() {
            Method::POST => login(req, state).await,
            Method::OPTIONS => Ok(Response::builder()
                .status(204)
                .body(Body::from("No Content"))
                .unwrap()),

            _ => Err(Error::MethodNotAllowed),
        },
        Some("refresh") => match *req.method() {
            Method::POST => refresh(req, state).await,
            Method::OPTIONS => Ok(Response::builder()
                .status(204)
                .body(Body::from("No Content"))
                .unwrap()),
            _ => Err(Error::MethodNotAllowed),
        },
        _ => Err(Error::NotFound),
    }
}

async fn login(req: Request, state: State) -> Result<Response<Body>, Error> {
    let data = req.json().await?;

    let mut resp = Response::new(Body::empty());

    if state.is_allowed(&data) {
        let tokens = create_token_pair(Claims::new(0))?;

        *resp.status_mut() = StatusCode::OK;
        *resp.body_mut() = Body::from(serde_json::to_vec(&tokens)?);
    } else {
        *resp.status_mut() = StatusCode::UNAUTHORIZED;
        *resp.body_mut() = Body::from("Unauthorized");
    }

    Ok(resp)
}

async fn refresh(req: Request, state: State) -> Result<Response<Body>, Error> {
    let body: RefreshToken = req.json().await?;

    let mut resp = Response::new(Body::empty());

    let claims = match state.decode_token(&body.refresh_token) {
        Ok(claims) => claims,
        Err(err) => {
            log::info!("Failed to decode jwt token: {:?}", err);

            *resp.status_mut() = StatusCode::UNAUTHORIZED;
            *resp.body_mut() = Body::from("Unauthorized");
            return Ok(resp);
        }
    };

    let body = create_token_pair(claims)?;

    *resp.status_mut() = StatusCode::OK;
    *resp.body_mut() = Body::from(serde_json::to_vec(&body)?);

    Ok(resp)
}

fn create_token_pair(mut claims: Claims) -> Result<TokenPair, jsonwebtoken::errors::Error> {
    let now = Utc::now().timestamp() as u64;

    claims.iat = now;
    claims.nbf = now;
    claims.exp = now + AUTH_TOKEN_EXP;

    let header = Header::default();
    let key = EncodingKey::from_secret(SECRET);

    let auth_token = jsonwebtoken::encode(&header, &claims, &key)?;

    claims.exp = now + REFRESH_TOKEN_EXPR;

    let refresh_token = jsonwebtoken::encode(&header, &claims, &key)?;

    Ok(TokenPair {
        auth_token,
        refresh_token,
    })
}
