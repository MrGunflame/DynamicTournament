use dynamic_tournament_generator::{Entrant, EntrantScore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Authorize(String),
    UpdateMatch {
        index: u64,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch {
        index: usize,
    },
    Close,
}

impl Message {
    pub fn into_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(buf: &[u8]) -> bincode::Result<Self> {
        bincode::deserialize(buf)
    }
}
