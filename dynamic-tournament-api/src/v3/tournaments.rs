use super::systems::SystemId;
use crate::{Client, Result};

use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A unique identifier for a [`Tournament`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct TournamentId(pub u64);

impl Display for TournamentId {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RoleId(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub description: String,
    /// RFC3339
    pub date: DateTime<Utc>,
    #[serde(default)]
    pub brackets: Vec<Bracket>,
    pub entrants: Entrants,
}

/// A list of entrants in a [`Tournament`]. `Entrants` can either be a list of [`Player`]s or a
/// list of [`Team`]s.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entrants {
    Players(Vec<Player>),
    Teams(Vec<Team>),
}

impl Entrants {
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Players(vec) => vec.len(),
            Self::Teams(vec) => vec.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct BracketId(pub u64);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bracket {
    pub id: BracketId,
    pub system: SystemId,
    /// The list of entrants playing in this bracket. All entrants must exist in the tournament.
    pub entrants: Vec<u64>,
    /// A list of optional key-value pairs for the bracket nodes.
    pub nodes: HashMap<String, NodeKind>,
}

/// All types avaliable to use for custom node values. For the value variant see [`NodeValue`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Bool,
    I64,
    U64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NodeValue {
    Bool(bool),
    I64(i64),
    U64(u64),
}

/// A single player in a tournament, either alone or as part of a [`Team`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub role: RoleId,
    pub rating: Option<u64>,
}

/// A single entrant for Team tournaments.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
}

pub struct TournamentsClient<'a> {
    client: &'a Client,
}

impl<'a> TournamentsClient<'a> {
    /// Returns the [`Tournament`] with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get(&self, id: TournamentId) -> Result<Tournament> {
        let req = self
            .client
            .request()
            .uri(&format!("/v3/tournaments/{}", id))
            .build();

        self.client.send(req).await?.json().await
    }

    /// Creates a new [`Tournament`].
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn create(&self, tournament: &Tournament) -> Result<()> {
        let req = self
            .client
            .request()
            .post()
            .uri("/v2/tournaments")
            .body(tournament)
            .build();

        self.client.send(req).await?;
        Ok(())
    }
}
