use serde::de::{self, Deserializer, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

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

    /// Consumes this `Token`, returning the raw parts.
    ///
    /// The parts can be used to reconstruct the `Token` using [`Token::from_parts`].
    pub fn into_parts(self) -> (String, Claims) {
        (self.token, self.claims)
    }

    /// Returns an reference to the JWT token.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Returns a reference to the claims of the JWT token.
    pub fn claims(&self) -> &Claims {
        &self.claims
    }

    /// Consumes this `Token`, returning only the token.
    pub fn into_token(self) -> String {
        self.token
    }

    /// Consumes this `Token`, returning only the [`Claims`].
    pub fn into_claims(self) -> Claims {
        self.claims
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
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.token.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Token {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TokenVisitor;

        impl<'de> Visitor<'de> for TokenVisitor {
            type Value = Token;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("token")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match Token::new(v) {
                    Ok(token) => Ok(token),
                    Err(err) => Err(E::custom(err)),
                }
            }

            #[inline]
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

impl AsRef<str> for Token {
    fn as_ref(&self) -> &str {
        self.token()
    }
}

/// The claims set on a [`Token`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Claims {
    pub sub: u64,
    pub iat: u64,
    pub exp: u64,
    pub nbf: u64,
    #[serde(default)]
    pub flags: Flags,
}

impl Claims {
    /// Creates a new set of `Claims` with `sub` set and all other fields set to their default.
    ///
    /// To also use the default value for `sub` use the [`Default`] implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_api::auth::Claims;
    /// #
    /// let claims = Claims::new(12);
    /// assert_eq!(claims.sub, 12);
    /// assert_eq!(claims.iat, 0);
    /// ```
    pub fn new(sub: u64) -> Self {
        Self {
            sub,
            iat: 0,
            exp: 0,
            nbf: 0,
            flags: Flags::new(),
        }
    }
}

/// A list of flags assigned to a token. This is represented as bitflags.
///
/// Use `|` ([`BitOr`]) to combine flags (union) and `&` ([`BitAnd`]) to intersect flags.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flags(u8);

impl Flags {
    /// An empty flags bitmap.
    pub const EMPTY: Self = Self(0);
    /// Whether the token has admin access. This includes all actions.
    pub const ADMIN: Self = Self(1);
    /// Whether the token has access to editing bracket scores.
    pub const EDIT_SCORES: Self = Self(1 << 1);

    /// All flags.
    pub const ALL: Self = Self(u8::MAX);

    /// Creates a new `Flags` bitmap with all flags disabled.
    #[inline]
    pub fn new() -> Self {
        Flags(0)
    }

    /// Returns `true` if the flags intersect with `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use dynamic_tournament_api::auth::Flags;
    /// #
    /// let flags = Flags::ADMIN | Flags::EDIT_SCORES;
    /// assert!(flags.intersects(Flags::ADMIN));
    /// assert!(flags.intersects(Flags::EDIT_SCORES));
    ///
    /// let flags = Flags::EDIT_SCORES;
    /// assert!(flags.intersects(Flags::EDIT_SCORES));
    /// assert!(!flags.intersects(Flags::ADMIN));
    /// ```
    #[inline]
    pub fn intersects(self, other: Self) -> bool {
        self & other == other
    }
}

impl BitAnd for Flags {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Flags {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Flags {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Flags {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

#[cfg(test)]
mod tests {
    use super::{Claims, Flags, Token};
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
                flags: Flags::new(),
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
                    flags: Flags::new(),
                },
            },
            &[SerToken::Str(token)],
        );
    }

    #[test]
    fn test_flags() {
        let mut flags = Flags::new();
        assert_eq!(flags.0, 0);

        flags |= Flags::ADMIN;
        assert_eq!(flags.0, Flags::ADMIN.0);
        assert_eq!(flags & Flags::ADMIN, Flags::ADMIN);

        flags |= Flags::EDIT_SCORES;
        assert_eq!(flags.0, (Flags::ADMIN | Flags::EDIT_SCORES).0);
        assert_eq!(
            flags & (Flags::ADMIN | Flags::EDIT_SCORES),
            (Flags::ADMIN | Flags::EDIT_SCORES),
        );
    }

    #[test]
    fn test_flags_intersects() {
        let flags = Flags::EMPTY;
        assert!(!flags.intersects(Flags::ADMIN));
        assert!(!flags.intersects(Flags::ALL));

        let flags = Flags::ALL;
        assert!(flags.intersects(Flags::ADMIN));
        assert!(flags.intersects(Flags::EDIT_SCORES));
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub auth_token: Token,
    pub refresh_token: Token,
}
