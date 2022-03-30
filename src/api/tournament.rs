use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use std::fmt::{self, Display, Formatter};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tournament {
    pub teams: Vec<Team>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Team {
    pub name: String,
    pub players: Vec<Player>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Player {
    #[serde(rename = "accountName")]
    pub account_name: String,
    pub role: Role,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum Role {
    Unknown = 0,
    Roamer = 1,
    Teamfighter = 2,
    Duelist = 3,
    Support = 4,
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
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
