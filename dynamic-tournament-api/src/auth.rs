use serde::{Deserialize, Serialize};

use crate::{Client, Error, Result};

pub struct AuthClient<'a> {
    client: &'a Client,
}

impl<'a> AuthClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let body = &LoginData {
            username: username.to_owned(),
            password: password.to_owned(),
        };

        let req = self
            .client
            .request()
            .post()
            .url("/v1/auth/login")
            .body(body);

        let resp = req.build().send().await?;

        if resp.ok() {
            let auth = format!(
                "Basic {}",
                base64::encode(&format!("{}:{}", username, password))
            );

            let mut inner = self.client.inner.write().unwrap();

            inner.authorization.update(Some(auth));

            Ok(())
        } else {
            Err(Error::BadStatusCode(resp.status()).into())
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct LoginData {
    username: String,
    password: String,
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

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("invalid token")]
    InvalidToken,
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("json decode error: {0}")]
    Json(#[from] serde_json::Error),
}
