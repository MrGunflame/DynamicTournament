use crate::{Client, Error, Result};

pub struct AuthClient<'a> {
    client: &'a Client,
}

impl<'a> AuthClient<'a> {
    pub async fn login(&self, username: &str, password: &str) -> Result<()> {
        let auth = format!(
            "Basic {}",
            base64::encode(&format!("{}:{}", username, password))
        );

        let req = self
            .client
            .request()
            .url("/v1/auth/login")
            .header("Authorization", auth);

        let resp = req.build().send().await?;

        if !resp.ok() {
            Err(Error::BadStatusCode(resp.status()).into())
        } else {
            Ok(())
        }
    }
}
