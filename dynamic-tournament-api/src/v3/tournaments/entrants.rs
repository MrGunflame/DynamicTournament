use crate::v3::id::{EntrantId, RoleId, TournamentId};
use crate::{Client, Result};

use serde::{Deserialize, Serialize};

use super::EntrantKind;

/// A single entrant. Depending on the [`EntrantKind`] of the tournament this is either
/// a [`Player`] or a [`Team`].
///
/// Note that a tournament can only ever have [`Player`]s **or** [`Team`]s.
///
/// [`EntrantKind`]: super::EntrantKind
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntrantVariant {
    Player(Player),
    Team(Team),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entrant {
    #[cfg_attr(feature = "server", serde(skip_deserializing))]
    pub id: EntrantId,
    #[serde(flatten)]
    pub inner: EntrantVariant,
}

impl Entrant {
    pub fn kind(&self) -> EntrantKind {
        match self.inner {
            EntrantVariant::Player(_) => EntrantKind::Player,
            EntrantVariant::Team(_) => EntrantKind::Team,
        }
    }
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

#[derive(Clone, Debug)]
pub struct EntrantsClient<'a> {
    client: &'a Client,
    tournament_id: TournamentId,
}

impl<'a> EntrantsClient<'a> {
    pub(crate) fn new(client: &'a Client, tournament_id: TournamentId) -> Self {
        Self {
            client,
            tournament_id,
        }
    }

    /// Returns a list of all entrant in the tournament.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn list(&self) -> Result<Vec<Entrant>> {
        let req = self
            .client
            .request()
            .uri(&format!("/v3/tournaments/{}/entrants", self.tournament_id))
            .build();

        self.client.send(req).await?.json().await
    }

    /// Returns the [`Entrant`] with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get(&self, id: EntrantId) -> Result<Entrant> {
        let req = self
            .client
            .request()
            .uri(&format!(
                "/v3/tournaments/{}/entrants/{}",
                self.tournament_id, id
            ))
            .build();

        self.client.send(req).await?.json().await
    }

    /// Creates a new [`Entrant`] for the tournament. Note that this returns an error if the
    /// incorrect [`Entrant`] variant is provided for the [`EntrantKind`] value of the tournaemnt.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn create(&self, entrant: &Entrant) -> Result<()> {
        let req = self
            .client
            .request()
            .uri(&format!("/v3/tournaments/{}/entrants", self.tournament_id))
            .post()
            .body(entrant)
            .build();

        self.client.send(req).await?.json().await
    }

    pub async fn delete(&self, id: EntrantId) -> Result<()> {
        let req = self
            .client
            .request()
            .uri(&format!(
                "/v3/tournaments/{}/entrants/{}",
                self.tournament_id, id
            ))
            .delete()
            .build();

        self.client.send(req).await?;
        Ok(())
    }
}
