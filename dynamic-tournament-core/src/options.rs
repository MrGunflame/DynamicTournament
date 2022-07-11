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
use std::hint;

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

    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: ToString,
        V: Into<OptionValue>,
    {
        let key = key.to_string();

        match self.0.get_mut(&key) {
            Some(val) => val.value = value.into(),
            None => {
                self.0.insert(
                    key,
                    TournamentOption {
                        name: String::new(),
                        value: value.into(),
                    },
                );
            }
        }
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

impl From<TournamentOptions> for TournamentOptionValues {
    fn from(this: TournamentOptions) -> Self {
        TournamentOptionValues::default().merge(this).unwrap()
    }
}

/// A list of optional key-values for a tournament which only contains the values.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TournamentOptionValues(HashMap<String, OptionValue>);

impl TournamentOptionValues {
    /// Returns the [`OptionValue`] with the given `key`. Returns `None` if no value exist for the
    /// given `key`.
    pub fn get(&self, key: &str) -> Option<&OptionValue> {
        self.0.get(key)
    }

    pub fn set<K, V>(&mut self, key: K, value: V)
    where
        K: ToString,
        V: Into<OptionValue>,
    {
        self.0.insert(key.to_string(), value.into());
    }

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
#[cfg_attr(feature = "serde", serde(untagged))]
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

    /// Returns the contained `Bool` value.
    ///
    /// # Panics
    ///
    /// Panics if the `self` value is not [`Bool`].
    ///
    /// [`Bool`]: Self::Bool
    #[inline]
    pub fn unwrap_bool(&self) -> bool {
        match self {
            Self::Bool(val) => *val,
            _ => panic!(
                "called `OptionValue::unwrap_bool()` on a `{}` value",
                self.panic_string()
            ),
        }
    }

    /// Returns the contained `Bool` value without checking whether `self` is `Bool`.
    ///
    /// # Safety
    ///
    /// Calling this method on a value other than `Bool` is undefined behavoir.
    #[inline]
    pub const unsafe fn unwrap_bool_unchecked(&self) -> bool {
        match self {
            Self::Bool(val) => *val,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Returns the contained `Bool` value or a provided default.
    #[inline]
    pub fn unwrap_bool_or(&self, default: bool) -> bool {
        match self {
            Self::Bool(val) => *val,
            _ => default,
        }
    }

    /// Returns the `&str` that should be used for `self` in a panic string.
    #[inline]
    fn panic_string(&self) -> &'static str {
        match self {
            Self::Bool(_) => "Bool",
            Self::I64(_) => "I64",
            Self::U64(_) => "U64",
            Self::String(_) => "String",
        }
    }
}

impl From<bool> for OptionValue {
    #[inline]
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for OptionValue {
    #[inline]
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}

impl From<u64> for OptionValue {
    #[inline]
    fn from(value: u64) -> Self {
        Self::U64(value)
    }
}

impl<'a> From<&'a str> for OptionValue {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for OptionValue {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

/// A builder for [`TournamentOptions`].
#[derive(Clone, Debug, Default)]
pub struct Builder {
    options: TournamentOptions,
}

impl Builder {
    /// Adds a
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

    /// Consumes the `Builder`, returning the collected [`TournamentOptions`].
    #[inline]
    pub fn build(self) -> TournamentOptions {
        self.options
    }
}
