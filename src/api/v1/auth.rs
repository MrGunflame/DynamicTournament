use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

use crate::components::config_provider::Config;

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
            .header("Content-Type", "application/json")
            .send()
            .await?;

        if !resp.ok() {
            return Err(BadStatusCodeError {
                status: resp.status(),
            }
            .into());
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct BadStatusCodeError {
    status: u16,
}

impl std::fmt::Display for BadStatusCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bad status code: {}", self.status)
    }
}

impl std::error::Error for BadStatusCodeError {}
