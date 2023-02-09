use chrono::{DateTime, Utc};
use dynamic_tournament_core::EntrantScore;
use serde::{Deserialize, Serialize};

use crate::v3::id::{BracketId, EventId, TournamentId};
use crate::{Client, Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub id: EventId,
    pub date: DateTime<Utc>,
    /// The user that invoked this action.
    pub author: u64,
    #[serde(flatten)]
    pub body: LogEventBody,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "content")]
pub enum LogEventBody {
    UpdateMatch {
        bracket_id: BracketId,
        index: u64,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch {
        bracket_id: BracketId,
        index: u64,
    },
}

#[derive(Copy, Clone, Debug)]
pub struct EventLogClient<'a> {
    client: &'a Client,
    tournament_id: TournamentId,
}

impl<'a> EventLogClient<'a> {
    pub(crate) fn new(client: &'a Client, tournament_id: TournamentId) -> Self {
        Self {
            client,
            tournament_id,
        }
    }

    pub async fn list(&self) -> Result<Vec<LogEvent>> {
        let uri = format!("/v3/tournaments/{}/log", self.tournament_id);

        let req = self.client.request().get().uri(&uri).build();
        let resp = self.client.send(req).await?;
        resp.json().await
    }
}
