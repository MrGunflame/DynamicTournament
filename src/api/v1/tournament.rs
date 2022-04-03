use gloo_storage::Storage;
use reqwasm::http::Request;
use serde::{Deserialize, Serialize};

use super::{auth::AuthCredentials, BadStatusCodeError};
use crate::{
    api::tournament::Team,
    bracket_generator::{EntrantWithScore, Match},
    components::config_provider::Config,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bracket(pub Vec<Match<EntrantWithScore<Team, u64>>>);

impl Bracket {
    pub async fn get(
        tournament_id: u64,
        config: Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let resp = Request::get(&format!(
            "{}/api/v1/tournament/{}/bracket",
            config.api_url, tournament_id
        ))
        .send()
        .await?
        .json()
        .await?;

        Ok(resp)
    }

    pub async fn put(
        &self,
        tournament_id: u64,
        config: Config,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let body = serde_json::to_string(&self.0).unwrap();

        let auth_data: AuthCredentials = gloo_storage::LocalStorage::get("http_auth_data")?;

        let resp = Request::put(&format!(
            "{}/api/v1/tournament/{}/bracket",
            config.api_url, tournament_id
        ))
        .header(
            "Authorization",
            &format!(
                "Basic {}",
                base64::encode(&format!("{}:{}", auth_data.username, auth_data.password))
            ),
        )
        .header("Content-Type", "application/json")
        .body(body)
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
