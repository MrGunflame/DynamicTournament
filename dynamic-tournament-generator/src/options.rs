//! # Tournament Options
//!
//! Some tournament [`System`]s may require additional and/or optional configuration that change
//! the behavoir of the [`System`]. An example would be including a match for the third place in a
//! single elimination tournament, or defining the rounds played in a swiss tournament.
//!
//! This module provides this kind of configuration via [`TournamentOption`] using a key-value map.
//! [`OptionValue`] contains all types supported.
//!
//! [`System`]: crate::System
use std::collections::{
    hash_map::{Iter, Keys},
    HashMap,
};

use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Error {
    #[error("missing key {0}")]
    MissingKey(String),
    #[error("unknown key {0}")]
    UnknownKey(String),
    #[error("invalid value for {key}: expected {expected}, found {found}")]
    InvalidValue {
        key: String,
        found: &'static str,
        expected: &'static str,
    },
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A list of optional values for a tournament.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TournamentOptions(HashMap<String, TournamentOption>);

impl TournamentOptions {
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the option with the given `key`. Returns `None` if the given key does not exist
    pub fn get(&self, key: &str) -> Option<&TournamentOption> {
        self.0.get(key)
    }

    pub fn insert<K>(&mut self, key: K, option: TournamentOption)
    where
        K: ToString,
    {
        self.0.insert(key.to_string(), option);
    }

    pub fn keys(&self) -> Keys<'_, String, TournamentOption> {
        self.0.keys()
    }

    pub fn iter(&self) -> Iter<'_, String, TournamentOption> {
        self.0.iter()
    }

    pub fn into_values(self) -> impl Iterator<Item = TournamentOption> {
        self.0.into_values()
    }
}

/// A list of optional key-values for a tournament which only contains the values.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TournamentOptionValues(HashMap<String, OptionValue>);

impl TournamentOptionValues {
    pub fn iter(&self) -> Iter<'_, String, OptionValue> {
        self.0.iter()
    }

    // TODO: Can avoid some `to_owned` calls.
    pub fn merge(mut self, mut options: TournamentOptions) -> Result<Self, Error> {
        for (key, value) in self.0.iter() {
            let default_value = match options.0.remove(key) {
                Some(value) => value,
                None => return Err(Error::UnknownKey(key.to_owned())),
            };

            if default_value.value.value_type() != value.value_type() {
                return Err(Error::InvalidValue {
                    key: key.to_owned(),
                    found: value.value_type(),
                    expected: default_value.value.value_type(),
                });
            }
        }

        // Fill the unassigned fields with defaults.
        for (key, value) in options.0.into_iter() {
            self.0.insert(key, value.value);
        }

        Ok(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TournamentOption {
    pub name: String,
    pub value: OptionValue,
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

impl OptionValue {
    /// Returns the name of the type of this value.
    pub fn value_type(&self) -> &'static str {
        match self {
            Self::Bool(_) => "bool",
            Self::I64(_) => "i64",
            Self::U64(_) => "u64",
            Self::String(_) => "string",
        }
    }

    pub fn unwrap_bool(&self) -> bool {
        match self {
            Self::Bool(value) => *value,
            _ => panic!("err"),
        }
    }
}

impl From<bool> for OptionValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for OptionValue {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u64> for OptionValue {
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl<'a> From<&'a str> for OptionValue {
    fn from(value: &'a str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for OptionValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Builder {
    options: TournamentOptions,
}

impl Builder {
    pub fn option<T, V>(mut self, key: &'static str, name: T, value: V) -> Self
    where
        T: ToString,
        V: Into<OptionValue>,
    {
        self.options.insert(
            key.to_string(),
            TournamentOption {
                name: name.to_string(),
                value: value.into(),
            },
        );
        self
    }

    pub fn build(self) -> TournamentOptions {
        self.options
    }
}
