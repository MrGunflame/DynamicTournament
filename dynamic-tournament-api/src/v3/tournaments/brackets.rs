use serde::{Deserialize, Serialize};

use crate::v3::id::{BracketId, EntrantId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bracket {
    #[cfg_attr(feature = "server", serde(skip_deserializing))]
    pub id: BracketId,
    /// An ordered list of the entrants playing in the bracket. Note that the order may be
    /// important and defines the initial placements if seeding is disabled.
    pub entrants: Vec<EntrantId>,
}
