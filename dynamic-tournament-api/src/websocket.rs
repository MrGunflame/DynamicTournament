use dynamic_tournament_generator::EntrantScore;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Reserved,
    Authorize(String),
    UpdateMatch {
        index: u64,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch {
        index: usize,
    },
}

impl Message {
    pub fn into_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }

    pub fn from_bytes(buf: &[u8]) -> bincode::Result<Self> {
        bincode::deserialize(buf)
    }
}
