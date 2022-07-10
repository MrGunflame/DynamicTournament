use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

use crate::Error;

/// A single JWT token.
#[derive(Clone, Debug)]
pub struct Token {
    token: String,
    claims: Claims,
}

impl Token {
    /// Creates a new `Token` from an JWT string.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if the JWT token is invalid.
    pub fn new<'a, T>(token: T) -> Result<Self, Error>
    where
        T: Into<Cow<'a, str>>,
    {
        let token = token.into();

        let claims = token.split('.').nth(1).ok_or(Error::InvalidToken)?;

        let claims = serde_json::from_slice(&base64::decode(claims)?)?;

        Ok(Self {
            token: token.into_owned(),
            claims,
        })
    }

    /// Creates a new `Token` using the raw parts. Note that `from_parts` does not check if the
    /// token is valid.
    pub fn from_parts(token: String, claims: Claims) -> Self {
        Self { token, claims }
    }

    /// Returns an reference to the JWT token.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns a reference to the claims of the JWT token.
    pub fn claims(&self) -> &Claims {
        &self.claims
    }
}

impl Display for Token {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.token.fmt(f)
    }
}

impl PartialEq for Token {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl Eq for Token {}

impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.token.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Token {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TokenVisitor;

        impl<'de> Visitor<'de> for TokenVisitor {
            type Value = Token;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("token")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Token::new(v) {
                    Ok(token) => Ok(token),
                    Err(err) => Err(E::custom(err)),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Token::new(v) {
                    Ok(token) => Ok(token),
                    Err(err) => Err(E::custom(err)),
                }
            }
        }

        deserializer.deserialize_string(TokenVisitor)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Claims {
    pub sub: u64,
    pub iat: u64,
    pub exp: u64,
    pub nbf: u64,
}

#[cfg(test)]
mod tests {
    use super::{Claims, Token};
    use crate::Error;

    use serde_test::{assert_tokens, Token as SerToken};

    #[test]
    fn test_token() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOjEyLCJpYXQiOjF9.2rAMFy3jpmaOhQ5jVygzSs4hS4hCIwuVDOk1hRmGgyI";

        let token = Token::new(token).unwrap();

        assert_eq!(
            token.claims,
            Claims {
                sub: 12,
                iat: 1,
                exp: 0,
                nbf: 0,
            }
        );

        let token = "invalid token";
        assert!(matches!(
            Token::new(token).unwrap_err(),
            Error::InvalidToken
        ));

        let token = "invalid.#.base64";
        assert!(matches!(Token::new(token).unwrap_err(), Error::Base64(_)));

        let token = "invalid.json.payload";
        assert!(matches!(
            Token::new(token).unwrap_err(),
            Error::SerdeJson(_)
        ));
    }

    #[test]
    fn test_token_serialize() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOjEyLCJpYXQiOjF9.2rAMFy3jpmaOhQ5jVygzSs4hS4hCIwuVDOk1hRmGgyI";

        assert_tokens(&Token::new(token).unwrap(), &[SerToken::Str(token)]);
    }

    #[test]
    fn test_token_deserialize() {
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOjEyLCJpYXQiOjF9.2rAMFy3jpmaOhQ5jVygzSs4hS4hCIwuVDOk1hRmGgyI";

        assert_tokens(
            &Token {
                token: token.to_string(),
                claims: Claims {
                    sub: 12,
                    iat: 1,
                    exp: 0,
                    nbf: 0,
                },
            },
            &[SerToken::Str(token)],
        );
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub auth_token: Token,
    pub refresh_token: Token,
}
