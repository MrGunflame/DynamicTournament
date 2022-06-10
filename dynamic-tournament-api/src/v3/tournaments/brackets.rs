pub mod matches;

use dynamic_tournament_generator::options::TournamentOptionValues;
use serde::{Deserialize, Serialize};

use crate::v3::id::{BracketId, EntrantId, SystemId, TournamentId};
use crate::websocket::{WebSocket, WebSocketBuilder};
use crate::{Client, Result};

use self::matches::Frame;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BracketOverview {
    pub id: BracketId,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bracket {
    #[cfg_attr(feature = "server", serde(skip_deserializing))]
    pub id: BracketId,
    pub name: String,
    pub system: SystemId,
    #[serde(default)]
    pub options: TournamentOptionValues,
    /// An ordered list of the entrants playing in the bracket. Note that the order may be
    /// important and defines the initial placements if seeding is disabled.
    pub entrants: Vec<EntrantId>,
}

#[derive(Clone, Debug)]
pub struct BracketsClient<'a> {
    client: &'a Client,
    tournament_id: TournamentId,
}

impl<'a> BracketsClient<'a> {
    pub(crate) fn new(client: &'a Client, tournament_id: TournamentId) -> Self {
        Self {
            client,
            tournament_id,
        }
    }

    pub async fn list(&self) -> Result<Vec<BracketOverview>> {
        let uri = format!("/v3/tournaments/{}/brackets", self.tournament_id);

        let req = self.client.request().get().uri(&uri).build();

        let resp = self.client.send(req).await?;

        resp.json().await
    }

    pub async fn get(&self, id: BracketId) -> Result<Bracket> {
        let uri = format!("/v3/tournaments/{}/brackets/{}", self.tournament_id, id);

        let req = self.client.request().get().uri(&uri).build();

        let resp = self.client.send(req).await?;

        resp.json().await
    }

    pub async fn create(&self, bracket: &Bracket) -> Result<()> {
        let uri = format!("/v3/tournaments/{}/brackets", self.tournament_id);

        let req = self.client.request().post().uri(&uri).body(bracket).build();

        self.client.send(req).await?.json().await
    }

    pub fn matches(&self, id: BracketId) -> WebSocketBuilder<Frame> {
        let uri = format!(
            "/v3/tournaments/{}/brackets/{}/matches",
            self.tournament_id, id
        );

        WebSocketBuilder::new(uri)
    }
}
