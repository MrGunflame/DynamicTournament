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

/// A list of optional values for a tournament. `TournamentOptions` includes the names and should
/// be used to describe a list of options. [`TournamentOptionValues`] should be used when just
/// expecting a list of key-value pairs.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TournamentOptions(HashMap<String, TournamentOption>);

impl TournamentOptions {
    /// Creates a new [`Builder`].
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

    /// Inserts a new [`TournamentOption`] with the provided `key`, overwriting the previous value
    /// if it exists.
    pub fn insert<K>(&mut self, key: K, option: TournamentOption)
    where
        K: ToString,
    {
        self.0.insert(key.to_string(), option);
    }

    /// Returns an iterator over all keys.
    pub fn keys(&self) -> Keys<'_, String, TournamentOption> {
        self.0.keys()
    }

    /// Returns an iterator over all [`TournamentOption`]s.
    pub fn iter(&self) -> Iter<'_, String, TournamentOption> {
        self.0.iter()
    }

    /// Consumes the `TournamentOptions`, returing an owned iterator over all
    /// [`TournamentOption`]s.
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

    pub fn take(&mut self, key: &str) -> Option<OptionValue> {
        self.0.remove(key)
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

/// The value of a [`TournamentOption`].
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

    /// Returns the contained [`Bool`] value.
    ///
    /// # Panics
    ///
    /// Panics if the `self` value is not [`Bool`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(true);
    /// assert!(val.unwrap_bool());
    /// ```
    ///
    /// ```should_panic
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::U64(0);
    /// assert!(val.unwrap_bool()); // Panics
    /// ```
    ///
    /// [`Bool`]: Self::Bool
    #[inline]
    pub fn unwrap_bool(self) -> bool {
        match self {
            Self::Bool(val) => val,
            _ => panic!(
                "called `OptionValue::unwrap_bool()` on a `{}` value",
                self.panic_string()
            ),
        }
    }

    /// Returns the contained [`Bool`] value without checking whether `self` is [`Bool`].
    ///
    /// # Safety
    ///
    /// Calling this method on a value other than [`Bool`] is undefined behavoir.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(true);
    /// assert!(unsafe { val.unwrap_bool_unchecked() });
    /// ```
    ///
    /// ```no_run
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::U64(0);
    /// assert!(unsafe { val.unwrap_bool_unchecked() }); // Undefined behavoir
    /// ```
    ///
    /// [`Bool`]: Self::Bool
    #[inline]
    pub unsafe fn unwrap_bool_unchecked(self) -> bool {
        match self {
            Self::Bool(val) => val,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Returns the contained [`Bool`] value or the provided default.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(true);
    /// assert!(val.unwrap_bool_or(false));
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::U64(0);
    /// assert!(val.unwrap_bool_or(true));
    /// ```
    ///
    /// [`Bool`]: Self::Bool
    #[inline]
    pub fn unwrap_bool_or(self, default: bool) -> bool {
        match self {
            Self::Bool(val) => val,
            _ => default,
        }
    }

    /// Returns the contained [`Bool`] value or computes it from the provided closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(true);
    /// assert!(val.unwrap_bool_or_else(|| false));
    /// ```
    ///
    /// [`Bool`]: Self::Bool
    #[inline]
    pub fn unwrap_bool_or_else<F>(self, f: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        match self {
            Self::Bool(val) => val,
            _ => f(),
        }
    }

    /// Returns the contained [`I64`] value.
    ///
    /// # Panics
    ///
    /// Panics if the `self` value is not [`I64`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::I64(-10);
    /// assert_eq!(val.unwrap_i64(), -10);
    /// ```
    ///
    /// ```should_panic
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_i64(), -10); // Panics
    /// ```
    ///
    /// [`I64`]: Self::I64
    #[inline]
    pub fn unwrap_i64(self) -> i64 {
        match self {
            Self::I64(val) => val,
            _ => panic!(
                "called `OptionValue::unwrap_i64` on a `{}` value",
                self.panic_string()
            ),
        }
    }

    /// Returns the contained [`I64`] value without checking whether `self` is [`I64`].
    ///
    /// # Safety
    ///
    /// Calling this method on a value other than [`I64`] is undefined behavoir.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::I64(-10);
    /// assert_eq!(unsafe { val.unwrap_i64_unchecked() }, -10);
    /// ```
    ///
    /// ```no_run
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(unsafe { val.unwrap_i64_unchecked() }, -10); // Undefined behavoir
    /// ```
    ///
    /// [`I64`]: Self::I64
    #[inline]
    pub unsafe fn unwrap_i64_unchecked(self) -> i64 {
        match self {
            Self::I64(val) => val,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Returns the contained [`I64`] value or the provided default.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::I64(-10);
    /// assert_eq!(val.unwrap_i64_or(0), -10);
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_i64_or(0), 0);
    /// ```
    ///
    /// [`I64`]: Self::I64
    #[inline]
    pub fn unwrap_i64_or(self, default: i64) -> i64 {
        match self {
            Self::I64(val) => val,
            _ => default,
        }
    }

    /// Returns the contained [`I64`] value or computes it from the provided closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::I64(-10);
    /// assert_eq!(val.unwrap_i64_or_else(|| 0), -10);
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_i64_or_else(|| 0), 0);
    /// ```
    ///
    /// [`I64`]: Self::I64
    #[inline]
    pub fn unwrap_i64_or_else<F>(self, f: F) -> i64
    where
        F: FnOnce() -> i64,
    {
        match self {
            Self::I64(val) => val,
            _ => f(),
        }
    }

    /// Returns the contained [`U64`] value.
    ///
    /// # Panics
    ///
    /// Panics if the `self` value is not [`U64`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::U64(5);
    /// assert_eq!(val.unwrap_u64(), 5);
    /// ```
    ///
    /// ```should_panic
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_u64(), 5); // Panics
    /// ```
    ///
    /// [`U64`]: Self::U64
    #[inline]
    pub fn unwrap_u64(self) -> u64 {
        match self {
            Self::U64(val) => val,
            _ => panic!(
                "called `OptionValue::unwrap_u64` on a `{}` value",
                self.panic_string()
            ),
        }
    }

    /// Returns the contained [`U64`] value without checking whether `self` is [`U64`].
    ///
    /// # Safety
    ///
    /// Calling this method on a value other than [`U64`] is undefined behavoir.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::U64(5);
    /// assert_eq!(unsafe { val.unwrap_u64_unchecked() }, 5);
    /// ```
    ///
    /// ```no_run
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(unsafe { val.unwrap_u64_unchecked() }, 5); // Undefined behavoir
    /// ```
    ///
    /// [`U64`]: Self::U64
    #[inline]
    pub unsafe fn unwrap_u64_unchecked(self) -> u64 {
        match self {
            Self::U64(val) => val,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Returns the contained [`U64`] value or the provided default.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::U64(5);
    /// assert_eq!(val.unwrap_u64_or(0), 5);
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_u64_or(0), 0);
    /// ```
    ///
    /// [`U64`]: Self::U64
    #[inline]
    pub fn unwrap_u64_or(self, default: u64) -> u64 {
        match self {
            Self::U64(val) => val,
            _ => default,
        }
    }

    /// Returns the contained [`U64`] value or computes it from the provided closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::U64(5);
    /// assert_eq!(val.unwrap_u64_or_else(|| 0), 5);
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_u64_or_else(|| 0), 0);
    /// ```
    ///
    /// [`U64`]: Self::U64
    #[inline]
    pub fn unwrap_u64_or_else<F>(self, f: F) -> u64
    where
        F: FnOnce() -> u64,
    {
        match self {
            Self::U64(val) => val,
            _ => f(),
        }
    }

    /// Returns the contained [`String`] value.
    ///
    /// # Panics
    ///
    /// Panics if the `self` value is not [`String`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::String(String::from("Hello"));
    /// assert_eq!(val.unwrap_string(), "Hello");
    /// ```
    ///
    /// ```should_panic
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_string(), "Hello"); // Panics
    /// ```
    ///
    /// [`String`]: Self::String
    #[inline]
    pub fn unwrap_string(self) -> String {
        match self {
            Self::String(val) => val,
            _ => panic!(
                "called `OptionValue::unwrap_string` on a `{}` value",
                self.panic_string()
            ),
        }
    }

    /// Returns the contained [`String`] value without checking whether `self` is [`String`].
    ///
    /// # Safety
    ///
    /// Calling this method on a value other than [`String`] is undefined behavoir.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::String(String::from("Hello"));
    /// assert_eq!(unsafe { val.unwrap_string_unchecked() }, "Hello");
    /// ```
    ///
    /// ```no_run
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(unsafe { val.unwrap_string_unchecked() }, "Hello"); // Undefined behavoir
    /// ```
    ///
    /// [`String`]: Self::String
    #[inline]
    pub unsafe fn unwrap_string_unchecked(self) -> String {
        match self {
            Self::String(val) => val,
            _ => hint::unreachable_unchecked(),
        }
    }

    /// Returns the contained [`String`] value or the provided default.
    ///
    /// Note that `default` is eagerly evaluated. You should use [`unwrap_string_or_else`] unless
    /// you already have a [`String`](String) to pass into `unwrap_string_or`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::String(String::from("Hello"));
    /// assert_eq!(val.unwrap_string_or(String::from("Hi")), "Hello");
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_string_or(String::from("Hi")), "Hi");
    /// ```
    ///
    /// [`String`]: Self::String
    /// [`unwrap_string_or_else`]: Self::unwrap_string_or_else
    #[inline]
    pub fn unwrap_string_or(self, default: String) -> String {
        match self {
            Self::String(val) => val,
            _ => default,
        }
    }

    /// Returns the contained [`String`] or computes it from the provided closure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::String(String::from("Hello"));
    /// assert_eq!(val.unwrap_string_or_else(|| String::from("Hi")), "Hello");
    /// ```
    ///
    /// ```
    /// # use dynamic_tournament_core::options::OptionValue;
    /// let val = OptionValue::Bool(false);
    /// assert_eq!(val.unwrap_string_or_else(|| String::from("Hi")), "Hi");
    /// ```
    ///
    /// [`String`]: Self::String
    #[inline]
    pub fn unwrap_string_or_else<F>(self, f: F) -> String
    where
        F: FnOnce() -> String,
    {
        match self {
            Self::String(val) => val,
            _ => f(),
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
    /// Inserts a new [`TournamentOption`]. If the `key` already exists, it is overwritten.
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
