pub mod brackets;
pub mod entrants;
pub mod roles;

use self::{brackets::BracketsClient, entrants::EntrantsClient, roles::RolesClient};

use super::id::TournamentId;
use crate::{Client, Result};

use entrants::{Player, Team};

use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TournamentOverview {
    pub id: TournamentId,
    pub name: String,
    pub date: DateTime<Utc>,
    pub kind: EntrantKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tournament {
    #[cfg_attr(feature = "server", serde(skip_deserializing))]
    pub id: TournamentId,
    pub name: String,
    pub description: String,
    /// RFC3339
    pub date: DateTime<Utc>,
    pub kind: EntrantKind,
}

impl Tournament {
    /// Updates an existing `Tournament` by applying a [`PartialTournament`] patch on it.
    pub fn update(&mut self, patch: PartialTournament) {
        if let Some(name) = patch.name {
            self.name = name;
        }

        if let Some(description) = patch.description {
            self.description = description;
        }

        if let Some(date) = patch.date {
            self.date = date;
        }

        if let Some(kind) = patch.kind {
            self.kind = kind;
        }
    }
}

/// A [`Tournament`] with all optional fields. This is primarly useful for `PATCH` requestsPATCH requests.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PartialTournament {
    #[cfg_attr(feature = "server", serde(skip_deserializing))]
    pub id: Option<TournamentId>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub kind: Option<EntrantKind>,
}

/// The type of [`Entrant`]s accepted by the tournament.
///
/// [`Entrant`]: entrants::Entrant
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntrantKind {
    Player,
    Team,
}

impl EntrantKind {
    #[inline]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Player => 0,
            Self::Team => 1,
        }
    }

    #[inline]
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Player),
            1 => Some(Self::Team),
            _ => None,
        }
    }
}

impl Display for EntrantKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Player => "Player",
                Self::Team => "Team",
            }
        )
    }
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

pub struct TournamentsClient<'a> {
    client: &'a Client,
}

impl<'a> TournamentsClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Returns a list of tournaments
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn list(&self) -> Result<Vec<TournamentOverview>> {
        let req = self.client.request().uri("/v3/tournaments").build();

        self.client.send(req).await?.json().await
    }

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

    /// Deletes the tournament with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn delete(&self, id: TournamentId) -> Result<()> {
        let req = self
            .client
            .request()
            .delete()
            .uri(&format!("/v3/tournaments/{}", id))
            .build();

        self.client.send(req).await?.json().await
    }

    /// Updates the tournament with the given `id` using the given tournament. Returns the updated
    /// [`Tournament`].
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn patch(
        &self,
        id: TournamentId,
        tournament: &PartialTournament,
    ) -> Result<Tournament> {
        let req = self
            .client
            .request()
            .patch()
            .uri(&format!("/v3/tournaments/{}", id))
            .body(tournament)
            .build();

        self.client.send(req).await?.json().await
    }

    pub fn brackets(&self, tournament_id: TournamentId) -> BracketsClient {
        BracketsClient::new(self.client, tournament_id)
    }

    pub fn entrants(&self, tournament_id: TournamentId) -> EntrantsClient {
        EntrantsClient::new(self.client, tournament_id)
    }

    pub fn roles(&self, tournament_id: TournamentId) -> RolesClient {
        RolesClient::new(self.client, tournament_id)
    }
}
