use serde::{Deserialize, Serialize};

use super::id::UserId;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub id: UserId,
    pub username: String,
    pub password: String,
}

