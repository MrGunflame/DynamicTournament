use serde::{Deserialize, Serialize};

use crate::{Client, Error, Result};

pub struct AuthClient<'a> {
    client: &'a Client,
}

impl<'a> AuthClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Login in using the given credentials.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] when the request fails. Returns [`Error::Unauthorized`] when the
    /// the given credentials are incorrect.
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let body = &LoginData {
            username: username.to_owned(),
            password: password.to_owned(),
        };

        let req = self
            .client
            .request()
            .post()
            .url("/v2/auth/login")
            .body(body);

        let resp = req.build().send().await?;

        if resp.ok() {
            let body = resp.json().await?;

            let mut inner = self.client.inner.write().unwrap();

            inner.authorization.update(body);

            Ok(())
        } else {
            match resp.status() {
                401 => Err(Error::Unauthorized),
                status => Err(Error::BadStatusCode(status)),
            }
        }
    }

    /// Refresh the authorization token pair.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] when the request fails. Returns [`Error::Unauthorized`] if no
    /// refresh token is avaliable.
    pub async fn refresh(&self) -> Result<()> {
        let refresh_token = {
            let inner = self.client.inner.read().unwrap();

            match inner.authorization.refresh_token() {
                Some(token) => token.to_owned(),
                None => return Err(Error::Unauthorized),
            }
        };

        let body = RefreshToken { refresh_token };

        let req = self
            .client
            .request()
            .post()
            .url("/v2/auth/refresh")
            .body(&body)
            .build();

        let resp = req.send().await?;

        if resp.ok() {
            let body = resp.json().await?;

            let mut inner = self.client.inner.write().unwrap();

            inner.authorization.update(body);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

/// A pair of two tokens. The `auth_token` is used to make requests.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub auth_token: String,
    pub refresh_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub refresh_token: String,
}

#[derive(Clone, Debug)]
pub struct AuthToken {
    token: String,
    claims: Claims,
}

impl AuthToken {
    pub fn new(token: String) -> std::result::Result<Self, JwtError> {
        let claims = token.split_once('.').ok_or(JwtError::InvalidToken)?.1;
        let claims = base64::decode(claims)?;
        let claims = serde_json::from_slice(&claims)?;

        Ok(Self { token, claims })
    }

    /// Returns the token of the `AuthToken`.
    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn claims(&self) -> &Claims {
        &self.claims
    }
}

impl PartialEq for AuthToken {
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Claims {
    /// Subject
    pub sub: u64,
    /// Issued at
    pub iat: u64,
    /// Expiration time
    pub exp: u64,
    /// Not before time
    pub nbf: u64,
}

impl Claims {
    pub fn new(sub: u64) -> Self {
        Self {
            sub,
            iat: 0,
            exp: 0,
            nbf: 0,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("invalid token")]
    InvalidToken,
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("json decode error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]

pub struct Token {
    token: String,
}

impl Token {
    pub fn new<T>(token: T) -> Self
    where
        T: ToString,
    {
        Self {
            token: token.to_string(),
        }
    }

    pub fn claims(&self) -> Claims {
        let parts = self.token.split('.');

        let claims = parts.skip(1).next().unwrap();

        let claims = base64::decode(claims).unwrap();

        serde_json::from_slice(&claims).unwrap()
    }
}
