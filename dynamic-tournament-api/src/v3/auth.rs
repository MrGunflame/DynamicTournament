use crate::{Error, Result};

use std::fmt::{self, Display, Formatter};

use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshToken {
    refresh_token: Token,
}

/// A pair of an auth and refresh token.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub auth_token: Token,
    pub refresh_token: Token,
}

#[derive(Clone, Debug)]
pub struct Token {
    token: String,
    claims: Claims,
}

impl Token {
    pub fn new<T>(token: T) -> Result<Self>
    where
        T: ToString,
    {
        let token = token.to_string();

        let claims = token.split('.').nth(1).ok_or(Error::InvalidToken)?;

        let claims = serde_json::from_slice(&base64::decode(claims)?)?;

        Ok(Self { token, claims })
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn claims(&self) -> &Claims {
        &self.claims
    }
}

impl Serialize for Token {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.token.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Token {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TokenVisitor;

        impl<'de> Visitor<'de> for TokenVisitor {
            type Value = Token;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("token")
            }

            fn visit_string<E>(self, v: String) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Token::new(v) {
                    Ok(token) => Ok(token),
                    Err(err) => Err(E::custom(err)),
                }
            }

            fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_string(v.to_string())
            }
        }

        deserializer.deserialize_string(TokenVisitor)
    }
}

impl Display for Token {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.token.fmt(f)
    }
}

// Tokens are equal when the token strings are equal. There is no need to compare
// the claims.
impl PartialEq for Token {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.token == other.token
    }
}

impl Eq for Token {}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Claims {
    /// Subject
    pub sub: u64,
    /// Issued At
    pub iat: u64,
    /// Expiration time
    pub exp: u64,
    /// Not before time
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
