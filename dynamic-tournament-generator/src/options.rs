use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A list of optional values for a tournament.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct TournamentOptions(HashMap<&'static str, TournamentOption>);

impl TournamentOptions {
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Returns the option with the given `key`. Returns `None` if the given key does not exist
    pub fn get(&self, key: &str) -> Option<&TournamentOption> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: &'static str, option: TournamentOption) {
        self.0.insert(key, option);
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
    pub fn value_type(&self) -> &str {
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
            key,
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
