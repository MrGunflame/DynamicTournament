//! serde impls for src/options.rs
use std::fmt::{self, Formatter};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::OptionValue;

impl Serialize for OptionValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::I64(v) => serializer.serialize_i64(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::String(v) => serializer.serialize_str(v),
        }
    }
}

impl<'de> Deserialize<'de> for OptionValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(OptionValueVisitor)
    }
}

struct OptionValueVisitor;

impl<'de> Visitor<'de> for OptionValueVisitor {
    type Value = OptionValue;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("a bool, i64, u64 or string")
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(OptionValue::Bool(v))
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(OptionValue::I64(v))
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(OptionValue::U64(v))
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(OptionValue::String(v.to_owned()))
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(OptionValue::String(v))
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};

    use super::OptionValue;

    #[test]
    fn test_option_value_serde() {
        assert_tokens(&OptionValue::Bool(true), &[Token::Bool(true)]);
        assert_tokens(&OptionValue::U64(123), &[Token::U64(123)]);
        assert_tokens(&OptionValue::I64(-456), &[Token::I64(-456)]);
        assert_tokens(&OptionValue::string("Hi"), &[Token::Str("Hi")]);
    }
}
