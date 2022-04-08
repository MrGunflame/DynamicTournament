use crate::{Client, Result};

use reqwasm::http::Request;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use std::fmt::{self, Display, Formatter};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TournamentId(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub bracket_type: BracketType,
    pub best_of: u64,
    pub teams: Vec<Team>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BracketType {
    SingleElimination = 0,
    DoubleElimination = 1,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    #[serde(rename = "accountName")]
    pub account_name: String,
    pub role: Role,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Role {
    Unknown = 0,
    Roamer = 1,
    Teamfighter = 2,
    Duelist = 3,
    Support = 4,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Unknown => "Unknown",
                Self::Roamer => "Roamer",
                Self::Teamfighter => "Teamfighter",
                Self::Duelist => "Duelist",
                Self::Support => "Support",
            }
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Tournaments<'a> {
    client: &'a Client,
}

impl<'a> Tournaments<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(&self) -> Result<Vec<TournamentId>> {
        let url = format!("{}/tournament", self.client.base_url);

        let resp = Request::get(&url).send().await?.json().await?;

        Ok(resp)
    }

    pub async fn get(&self, id: TournamentId) -> Result<Tournament> {
        let url = format!("{}/tournament/{}", self.client.base_url, id.0);

        let resp = Request::get(&url).send().await?.json().await?;

        Ok(resp)
    }
}
