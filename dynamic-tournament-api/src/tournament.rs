use crate::{Client, Result};

use dynamic_tournament_generator::{EntrantScore, Matches};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use std::fmt::{self, Display, Formatter};

// //////////////////////
// /// /v2/tournament ///
// //////////////////////

/// A unique identifier for a [`Tournament`].
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct TournamentId(pub u64);

impl Display for TournamentId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TournamentOverview {
    pub id: TournamentId,
    pub name: String,
    /// RFC3339
    pub date: DateTime<Utc>,
    pub bracket_type: BracketType,
    pub entrants: u64,
}

/// Full data about a tournament.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tournament {
    #[cfg_attr(feature = "server", serde(skip_deserializing))]
    pub id: TournamentId,
    pub name: String,
    pub description: String,
    /// RFC3339
    pub date: DateTime<Utc>,
    pub bracket_type: BracketType,
    pub entrants: Entrants,
}

/// The type of the bracket of a [`Tournament`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum BracketType {
    SingleElimination = 0,
    DoubleElimination = 1,
}

impl Display for BracketType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let string = match self {
            Self::SingleElimination => "Single Elimination",
            Self::DoubleElimination => "Double Elimination",
        };

        write!(f, "{}", string)
    }
}

impl TryFrom<u8> for BracketType {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::SingleElimination),
            1 => Ok(Self::DoubleElimination),
            _ => Err(()),
        }
    }
}

impl From<BracketType> for u8 {
    fn from(t: BracketType) -> Self {
        match t {
            BracketType::SingleElimination => 0,
            BracketType::DoubleElimination => 1,
        }
    }
}

/// A single team playing in a [`Tournament`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
}

impl Display for Team {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.name.fmt(f)
    }
}

/// A single player in a [`Team`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub role: Role,
    /// Rating of the player.
    #[serde(default)]
    pub rating: Option<u64>,
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}

/// The role of a [`Player`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[repr(u8)]
pub enum Role {
    Unknown = 0,
    Roamer = 1,
    Teamfighter = 2,
    Duelist = 3,
    Support = 4,
}

impl TryFrom<u8> for Role {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Roamer),
            2 => Ok(Self::Teamfighter),
            3 => Ok(Self::Duelist),
            4 => Ok(Self::Support),
            _ => Err(()),
        }
    }
}

impl From<Role> for u8 {
    fn from(role: Role) -> Self {
        match role {
            Role::Unknown => 0,
            Role::Roamer => 1,
            Role::Teamfighter => 2,
            Role::Duelist => 3,
            Role::Support => 4,
        }
    }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entrants {
    #[serde(rename = "players")]
    Players(Vec<Player>),
    #[serde(rename = "teams")]
    Teams(Vec<Team>),
}

impl Entrants {
    pub fn len(&self) -> usize {
        match self {
            Self::Players(players) => players.len(),
            Self::Teams(teams) => teams.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn unwrap_players(self) -> Vec<Player> {
        match self {
            Self::Players(players) => players,
            _ => panic!("Called unwrap_players on `Entrants::Teams`"),
        }
    }

    pub fn unwrap_teams(self) -> Vec<Team> {
        match self {
            Self::Teams(teams) => teams,
            _ => panic!("Called unwrap_teams on `Entrants::Players`"),
        }
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
    pub async fn list(&self) -> Result<Vec<TournamentOverview>> {
        let req = self.client.request().uri("/v2/tournament").build();

        let resp = self.client.send(req).await?.json().await?;

        Ok(resp)
    }

    /// Returns a single [`Tournament`].
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, or the returned data is invalid.
    pub async fn get(&self, id: TournamentId) -> Result<Tournament> {
        let req = self
            .client
            .request()
            .uri(&format!("/v2/tournament/{}", id.0))
            .build();

        let resp = self.client.send(req).await?.json().await?;

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
            .uri("/v2/tournament")
            .post()
            .body(tournament)
            .build();

        self.client.send(req).await?;
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
// /// /v2/tournament/{id}/bracket ///
// ///////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bracket(pub Matches<EntrantScore<u64>>);

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
            .uri(&format!("/v2/tournament/{}/bracket", self.tournament_id.0))
            .build();

        let resp = self.client.send(req).await?.json().await?;

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
            .uri(&format!("/v2/tournament/{}/bracket", self.tournament_id.0))
            .put()
            .body(bracket)
            .build();

        self.client.send(req).await?;
        Ok(())
    }
}
