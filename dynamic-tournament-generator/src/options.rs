use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A list of optional values for a tournament.
#[derive(Clone, Debug)]
pub struct TournamentOptions(HashSet<TournamentOption>);

impl TournamentOptions {
    /// Returns the option with the given `key`. Returns `None` if the given key does not exist
    pub fn get(&self, key: &str) -> Option<&TournamentOption> {
        self.0.get(key)
    }

    pub fn insert(&mut self, option: TournamentOption) {
        self.0.insert(option);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TournamentOption {
    key: &'static str,
    name: &'static str,
    value: OptionValue,
}

impl PartialEq<str> for TournamentOption {
    fn eq(&self, other: &str) -> bool {
        self.key == other
    }
}

impl Borrow<str> for TournamentOption {
    fn borrow(&self) -> &str {
        self.key
    }
}

impl Hash for TournamentOption {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.key.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[serde(untagged)]
pub enum OptionValue {
    Bool(bool),
    I64(i64),
    U64(u64),
    String(String),
}
