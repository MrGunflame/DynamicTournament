use chrono::{DateTime, Utc};
use dynamic_tournament_core::EntrantScore;
use serde::{Deserialize, Serialize};

use crate::v3::id::{BracketId, LogEntryId};

use super::EntrantKind;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: LogEntryId,
    pub author: u64,
    pub event: Event,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    /// The tournament was created.
    CreateTournament {
        name: String,
        description: String,
        date: DateTime<Utc>,
        kind: EntrantKind,
    },
    /// The tournament was updated. Only includes changed fields.
    UpdateTournament {
        name: Option<String>,
        description: Option<String>,
        date: Option<DateTime<Utc>>,
        kind: Option<EntrantKind>,
    },
    UpdateMatch {
        bracket: BracketId,
        index: usize,
        nodes: Vec<EntrantScore<u64>>,
    },
    ResetMatch {
        bracket: BracketId,
        index: usize,
    },
}
