use crate::{Client, Error, Result};

pub struct AuthClient<'a> {
    client: &'a Client,
}

impl<'a> AuthClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let auth = format!(
            "Basic {}",
            base64::encode(&format!("{}:{}", username, password))
        );

        let req = self
            .client
            .request()
            .url("/v1/auth/login")
            .header("Authorization", auth.clone());

        let resp = req.build().send().await?;

        if resp.ok() {
            let mut inner = self.client.inner.write().unwrap();

            inner.authorization.update(Some(auth));

            Ok(())
        } else {
            Err(Error::BadStatusCode(resp.status()).into())
        }
    }
}
