use crate::auth::Token;
use crate::{Client, Error, Result};

use http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub struct AuthClient<'a> {
    client: &'a Client,
}

impl<'a> AuthClient<'a> {
    /// Creates a new `AuthClient`.
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn login(&self) -> Result<()> {
        let req = self.client.request().post().uri("/v3/auth/login").build();

        let resp = self.client.send(req).await?;

        if resp.is_success() {
            let body = resp.json().await?;

            let mut inner = self.client.inner.write().unwrap();

            inner.authorization.update(body);

            Ok(())
        } else {
            match resp.status() {
                StatusCode::UNAUTHORIZED => Err(Error::Unauthorized),
                status => Err(Error::BadStatusCode(status)),
            }
        }
    }

    /// Refresh the authorization token pair.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] when the request fails. Returns [`Error::Unauthorized`] if no
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
            .uri("/v3/auth/refresh")
            .body(&body)
            .build();

        let resp = self.client.send(req).await?;

        if resp.is_success() {
            let body = resp.json().await?;

            let mut inner = self.client.inner.write().unwrap();

            inner.authorization.update(body);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshToken {
    pub refresh_token: Token,
}
