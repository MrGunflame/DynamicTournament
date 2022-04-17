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
