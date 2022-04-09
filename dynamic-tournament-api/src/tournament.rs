use crate::{Client, Result};

use dynamic_tournament_generator::{EntrantWithScore, Match};

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use std::fmt::{self, Display, Formatter};

// //////////////////////
// /// /v1/tournament ///
// //////////////////////

/// A unique identifier for a [`Tournament`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TournamentId(pub u64);

/// Full data about a tournament.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tournament {
    pub id: TournamentId,
    pub name: String,
    pub bracket_type: BracketType,
    pub best_of: u64,
    pub teams: Vec<Team>,
}

/// The type of the bracket of a [`Tournament`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum BracketType {
    SingleElimination = 0,
    DoubleElimination = 1,
}

/// A single team playing in a [`Tournament`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
}

/// A single player in a [`Team`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    #[serde(rename = "accountName")]
    pub account_name: String,
    pub role: Role,
}

/// The role of a [`Player`].
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
pub struct TournamentClient<'a> {
    client: &'a Client,
}

impl<'a> TournamentClient<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Returns a list of all tournaments.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, or the returned data is invalid.
    pub async fn list(&self) -> Result<Vec<TournamentId>> {
        let req = self.client.request().url("/tournament");

        let resp = req.build().send().await?.json().await?;

        Ok(resp)
    }

    /// Returns a single [`Tournament`].
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, or the returned data is invalid.
    pub async fn get(&self, id: TournamentId) -> Result<Tournament> {
        let req = self.client.request().url(format!("/tournament/{}", id.0));

        let resp = req.build().send().await?.json().await?;

        Ok(resp)
    }

    /// Creates a new [`Tournament`].
    ///
    /// # Errors
    ///
    /// /// Returns an error if the request fails.
    pub async fn create(&self, tournament: &Tournament) -> Result<()> {
        let req = self
            .client
            .request()
            .url("/tournament")
            .post()
            .body(tournament);

        req.build().send().await?;
        Ok(())
    }

    /// Creates a new [`BracketClient`] which is used to query and update the bracket state.
    ///
    /// **Note:** Calling this method on an `id` value that does not exist on the server won't
    /// return an error until an actual request is made. Calling this method does not guarantee
    /// the given `id` value exists.
    pub fn bracket(&self, id: TournamentId) -> BracketClient<'_> {
        BracketClient {
            client: self.client,
            tournament_id: id,
        }
    }
}

// ///////////////////////////////////
// /// /v1/tournament/{id}/bracket ///
// ///////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bracket(pub Vec<Match<EntrantWithScore<Team, u64>>>);

#[derive(Copy, Clone, Debug)]
pub struct BracketClient<'a> {
    client: &'a Client,
    tournament_id: TournamentId,
}

impl<'a> BracketClient<'a> {
    /// Returns the [`Bracket`] of an [`Tournament`].
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, or the returned data is invalid.
    pub async fn get(&self) -> Result<Bracket> {
        let req = self
            .client
            .request()
            .url(format!("/tournament/{}/bracket", self.tournament_id.0));

        let resp = req.build().send().await?.json().await?;

        Ok(resp)
    }

    /// Updates the [`Bracket`] on the associated [`Tournament`].
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn put(&self, bracket: &Bracket) -> Result<()> {
        let req = self
            .client
            .request()
            .url(format!("/tournament/{}/bracket", self.tournament_id.0))
            .put()
            .body(bracket);

        req.build().send().await?;
        Ok(())
    }
}
