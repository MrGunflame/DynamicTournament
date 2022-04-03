use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

use super::BadStatusCodeError;
use crate::components::config_provider::Config;

use gloo_storage::{LocalStorage, Storage};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AuthCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginData {
    username: String,
    password: String,
}

impl LoginData {
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    pub async fn post(&self, config: Config) -> Result<(), Box<dyn std::error::Error>> {
        let body = serde_json::to_string(self).unwrap();

        let resp = Request::post(&format!("{}/api/v1/auth/login", config.api_url))
            .body(body)
            .header(
                "Authorization",
                &format!(
                    "Basic {}",
                    base64::encode(&format!("{}:{}", self.username, self.password)),
                ),
            )
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !resp.ok() {
            return Err(BadStatusCodeError {
                status: resp.status(),
            }
            .into());
        }

        LocalStorage::set("http_auth_data", self)?;

        Ok(())
    }
}
