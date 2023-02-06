use chrono::{DateTime, Utc};
use dynamic_tournament_core::EntrantScore;
use serde::{Deserialize, Serialize};

use crate::v3::id::{BracketId, EventId};

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
