use std::fmt::{self, Debug, Formatter};

use chrono::Utc;
use dynamic_tournament_api::auth::{Claims, Token, TokenPair};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::Error;

/// Auth token expiration time.
const AUTH_TOKEN_EXP: u64 = 60 * 60;
/// Refresh token expiration time.
const REFRESH_TOKEN_EXP: u64 = 60 * 60 * 24;

pub const SECRET: &[u8] = include_bytes!("../jwt-secret");

/// A utility type to handle all [`Token`] encoding, decoding and validating.
#[derive(Clone)]
pub struct Authorization {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl Authorization {
    /// Creates a new `Authorization` instance which uses given [`Algorithm`].
    #[inline]
    pub fn new(alg: Algorithm) -> Self {
        let mut validation = Validation::new(alg);
        validation.validate_exp = false;

        Self {
            encoding_key: EncodingKey::from_secret(SECRET),
            decoding_key: DecodingKey::from_secret(SECRET),
            validation,
        }
    }

    /// Generate a new [`TokenPair`] using the provided [`Claims`].
    ///
    /// Note that the `iat`, `nbf` and `exp` claims will be overwritten.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if encoding the new tokens fails.
    pub fn create_tokens(&self, mut claims: Claims) -> Result<TokenPair, Error> {
        let now = Utc::now().timestamp() as u64;

        // Generate auth token.
        claims.iat = now;
        claims.nbf = now;
        claims.exp = now + AUTH_TOKEN_EXP;
        let auth_token = self.encode_token(claims.clone())?;

        // Generate refresh token.
        claims.exp = now + REFRESH_TOKEN_EXP;
        let refresh_token = self.encode_token(claims)?;

        Ok(TokenPair {
            auth_token,
            refresh_token,
        })
    }

    /// Encodes a new [`Token`] using the provided [`Claims`].
    ///
    /// Note that this method will not modify the claims (for claims like `iat`, `exp`, etc..) and
    /// use the provided claims as they are. If you need to create a new token but want to add the
    /// correct claims, use [`Self::create_tokens`].
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if encoding the new token fails.
    pub fn encode_token(&self, claims: Claims) -> Result<Token, Error> {
        let header = Header::default();
        let token = jsonwebtoken::encode(&header, &claims, &self.encoding_key)?;
        Ok(Token::from_parts(token, claims))
    }

    /// Decodes and validates (signature) a [`Token`].
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if decoding the token fails. This can happen if the token is malformed
    /// or contains an invalid signature.
    pub fn decode_token<T>(&self, token: T) -> Result<Token, Error>
    where
        T: AsRef<str> + ToString,
    {
        let data = jsonwebtoken::decode(token.as_ref(), &self.decoding_key, &self.validation)?;
        Ok(Token::from_parts(token.to_string(), data.claims))
    }

    /// Decodes and validates an auth token, including all claims.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if decoding the token fails, any claims are invalid or the given token
    /// is not an auth token.
    pub fn validate_auth_token<T>(&self, token: T) -> Result<Token, Error>
    where
        T: AsRef<str> + ToString,
    {
        let token = self.decode_token(token)?;

        let now = Utc::now().timestamp() as u64;

        for claim in [token.claims().iat, token.claims().nbf, token.claims().exp] {
            if claim == 0 {
                return Err(Error::InvalidToken);
            }
        }

        if token.claims().exp < now {
            return Err(Error::InvalidToken);
        }

        if token.claims().exp - token.claims().nbf != AUTH_TOKEN_EXP {
            return Err(Error::InvalidToken);
        }

        Ok(token)
    }

    /// Decodes and validates a refresh token, including all claims.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if decoding the token fails, any claims are invalid or the given token
    /// is not a refresh token.
    pub fn validate_refresh_token<T>(&self, token: T) -> Result<Token, Error>
    where
        T: AsRef<str> + ToString,
    {
        let token = self.decode_token(token)?;

        let now = Utc::now().timestamp() as u64;

        for claim in [token.claims().iat, token.claims().nbf, token.claims().exp] {
            if claim == 0 {
                return Err(Error::InvalidToken);
            }
        }

        if token.claims().exp < now {
            return Err(Error::InvalidToken);
        }

        if token.claims().exp - token.claims().nbf != REFRESH_TOKEN_EXP {
            return Err(Error::InvalidToken);
        }

        Ok(token)
    }
}

impl Debug for Authorization {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Authorization {{ encoding_key, decoding_key }}")
    }
}

#[cfg(test)]
mod tests {
    use crate::auth::{AUTH_TOKEN_EXP, REFRESH_TOKEN_EXP};

    use super::Authorization;

    use chrono::Utc;
    use dynamic_tournament_api::auth::{Claims, Token};
    use jsonwebtoken::Algorithm;

    #[test]
    fn test_create_tokens() {
        let auth = Authorization::new(Algorithm::HS256);

        let claims = Claims {
            sub: 0,
            iat: 0,
            nbf: 0,
            exp: 0,
        };
        let tokens = auth.create_tokens(claims).unwrap();

        assert_eq!(
            tokens.auth_token.claims().exp - tokens.auth_token.claims().nbf,
            AUTH_TOKEN_EXP
        );

        assert_eq!(
            tokens.refresh_token.claims().exp - tokens.refresh_token.claims().nbf,
            REFRESH_TOKEN_EXP
        )
    }

    #[test]
    fn test_encode_token() {
        let auth = Authorization::new(Algorithm::HS256);

        let claims = Claims {
            sub: 0,
            iat: 0,
            nbf: 0,
            exp: 0,
        };
        let token = auth.encode_token(claims.clone()).unwrap();

        // Decode the token to check the actual claims in the token.
        let token = Token::new(token.into_token()).unwrap();
        assert_eq!(token.into_claims(), claims);
    }

    #[test]
    fn test_decode_token() {
        let auth = Authorization::new(Algorithm::HS256);

        let claims = Claims {
            sub: 0,
            iat: 0,
            nbf: 0,
            exp: 0,
        };
        let tokens = auth.create_tokens(claims).unwrap();

        auth.decode_token(tokens.auth_token).unwrap();
        auth.decode_token(tokens.refresh_token).unwrap();

        // Token with invalid signature.
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIwIiwiaWF0IjowfQ.aJgGcoLu-bVZxlmrKpOKb3gpRkn9QJL5m-My7hp2yUE";
        auth.decode_token(token).unwrap_err();
    }

    #[test]
    fn test_validate_auth_token() {
        let auth = Authorization::new(Algorithm::HS256);

        let claims = Claims {
            sub: 0,
            iat: 0,
            nbf: 0,
            exp: 0,
        };
        let tokens = auth.create_tokens(claims).unwrap();

        auth.validate_auth_token(tokens.auth_token).unwrap();
        auth.validate_auth_token(tokens.refresh_token).unwrap_err();

        // Valid token.
        let now = Utc::now().timestamp() as u64;
        let token = auth
            .encode_token(Claims {
                sub: 0,
                iat: now,
                nbf: now,
                exp: now + AUTH_TOKEN_EXP,
            })
            .unwrap();
        auth.validate_auth_token(token).unwrap();

        // Token with invalid iat, nbf, exp claim.
        let token = auth
            .encode_token(Claims {
                sub: 0,
                iat: 0,
                nbf: now,
                exp: now + AUTH_TOKEN_EXP,
            })
            .unwrap();
        auth.validate_auth_token(token).unwrap_err();

        // Expired token.
        let token = auth
            .encode_token(Claims {
                sub: 0,
                iat: now,
                nbf: now,
                exp: now + AUTH_TOKEN_EXP + 1,
            })
            .unwrap();
        auth.validate_auth_token(token).unwrap_err();

        // Token with invalid signature.
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIwIiwiaWF0IjowfQ.aJgGcoLu-bVZxlmrKpOKb3gpRkn9QJL5m-My7hp2yUE";
        auth.validate_auth_token(token).unwrap_err();
    }

    #[test]
    fn test_validate_refresh_token() {
        let auth = Authorization::new(Algorithm::HS256);

        let claims = Claims {
            sub: 0,
            iat: 0,
            nbf: 0,
            exp: 0,
        };
        let tokens = auth.create_tokens(claims).unwrap();

        auth.validate_refresh_token(tokens.auth_token).unwrap_err();
        auth.validate_refresh_token(tokens.refresh_token).unwrap();

        // Valid token.
        let now = Utc::now().timestamp() as u64;
        let token = auth
            .encode_token(Claims {
                sub: 0,
                iat: now,
                nbf: now,
                exp: now + REFRESH_TOKEN_EXP,
            })
            .unwrap();
        auth.validate_refresh_token(token).unwrap();

        // Token with invalid iat, nbf, exp claim.
        let token = auth
            .encode_token(Claims {
                sub: 0,
                iat: 0,
                nbf: now,
                exp: now + REFRESH_TOKEN_EXP,
            })
            .unwrap();
        auth.validate_refresh_token(token).unwrap_err();

        // Expired token.
        let token = auth
            .encode_token(Claims {
                sub: 0,
                iat: now,
                nbf: now,
                exp: now + REFRESH_TOKEN_EXP + 1,
            })
            .unwrap();
        auth.validate_refresh_token(token).unwrap_err();

        // Token with invalid signature.
        let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIwIiwiaWF0IjowfQ.aJgGcoLu-bVZxlmrKpOKb3gpRkn9QJL5m-My7hp2yUE";
        auth.validate_refresh_token(token).unwrap_err();
    }
}
