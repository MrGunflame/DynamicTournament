use std::env;
use std::fmt::{self, Formatter};
use std::io;
use std::net::{AddrParseError, SocketAddr};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use jsonwebtoken::Algorithm;
use log::LevelFilter;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

macro_rules! from_environment {
    ($config:expr, $($key:expr, $name:tt),*$(,)?) => {{
        $(
            {
                if let Ok(value) = env::var($key) {
                    if let Ok(value) = value.parse() {
                        $config.$name = value;
                    }
                }
            }
        )*
    }};
}

macro_rules! from_environment_error {
    ($config:expr, $($key:expr, $name:tt),*$(,)?) => {{
        $(
            let value = env::var($key).map_err(|_| ConfigError::MissingField($key))?;
            $config.$name = value.parse().map_err(|_| ConfigError::MissingField($key))?;
        )*
    }};
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub database: Database,
    pub loglevel: LevelFilter,
    pub bind: BindAddr,

    pub authorization: Authorization,
}

impl Config {
    pub async fn from_file<P>(path: P) -> Result<Self, ConfigError>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path).await?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;

        Ok(toml::from_slice(&buf)?)
    }

    /// Creates a complete [`Config`] instance from the environment.
    pub fn from_environment() -> Result<Self, ConfigError> {
        let mut this = Self::default();

        from_environment_error!(this, "DT_LOGLEVEL", loglevel, "DT_BIND", bind);

        this.database = Database::from_environment()?;
        this.authorization = Authorization::from_environment()?;

        Ok(this)
    }

    pub fn with_environment(mut self) -> Self {
        from_environment!(self, "DT_LOGLEVEL", loglevel, "DT_BIND", bind);
        self.database = self.database.with_environment();
        self.authorization = self.authorization.with_environment();

        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: Database::default(),
            loglevel: LevelFilter::Info,
            bind: BindAddr::Tcp(SocketAddr::new([0, 0, 0, 0].into(), 3000)),
            authorization: Authorization::default(),
        }
    }
}

/// An address to bind the http server to.
///
/// This can currently be a tcp socket (net) or a unix socket (file).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum BindAddr {
    Tcp(SocketAddr),
    Unix(PathBuf),
}

impl BindAddr {
    /// Parses the given string into a `Tcp` address.
    ///
    /// # Errors
    ///
    /// Returns an [`AddrParseError`] when parsing the input fails.
    #[inline]
    pub fn parse_socket(s: &str) -> Result<Self, AddrParseError> {
        s.parse().map(Self::Tcp)
    }
}

impl FromStr for BindAddr {
    type Err = AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(addr) = Self::parse_socket(s) {
            return Ok(addr);
        }

        Ok(Self::Unix(s.to_owned().into()))
    }
}

impl<'de> Deserialize<'de> for BindAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BindAddrVisitor;

        impl<'de> Visitor<'de> for BindAddrVisitor {
            type Value = BindAddr;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("an address with port, or file path")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match v.parse() {
                    Ok(addr) => Ok(addr),
                    Err(err) => Err(E::custom(err)),
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(BindAddrVisitor)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Database {
    pub driver: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub prefix: String,
}

impl Database {
    pub fn connect_string(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}?ssl-mode=DISABLED",
            self.driver, self.user, self.password, self.host, self.port, self.database
        )
    }

    pub fn from_environment() -> Result<Self, ConfigError> {
        let mut this = Self::default();

        from_environment_error!(
            this,
            "DT_DB_DRIVER",
            driver,
            "DT_DB_HOST",
            host,
            "DT_DB_PORT",
            port,
            "DT_DB_USER",
            user,
            "DT_DB_PASSWORD",
            password,
            "DT_DB_DATABASE",
            database,
            "DT_DB_PREFIX",
            prefix,
        );

        Ok(this)
    }

    pub fn with_environment(mut self) -> Self {
        from_environment!(
            self,
            "DT_DB_DRIVER",
            driver,
            "DT_DB_HOST",
            host,
            "DT_DB_PORT",
            port,
            "DT_DB_USER",
            user,
            "DT_DB_PASSWORD",
            password,
            "DT_DB_DATABASE",
            database,
            "DT_DB_PREFIX",
            prefix,
        );

        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Authorization {
    pub alg: Algorithm,
}

impl Authorization {
    pub fn from_environment() -> Result<Self, ConfigError> {
        let mut this = Self::default();

        from_environment!(this, "DT_AUTH_ALG", alg);

        Ok(this)
    }

    pub fn with_environment(mut self) -> Self {
        from_environment!(self, "DT_AUTH_ALG", alg);

        self
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Toml(#[from] toml::de::Error),
    #[error("missing config field: {0}")]
    MissingField(&'static str),
}

#[cfg(test)]
mod tests {
    use super::BindAddr;

    #[test]
    fn test_bindaddr_parse() {
        let input = "0.0.0.0:80";
        assert_eq!(
            input.parse::<BindAddr>().unwrap(),
            BindAddr::Tcp(input.parse().unwrap())
        );

        let input = "/var/run/test";
        assert_eq!(
            input.parse::<BindAddr>().unwrap(),
            BindAddr::Unix(input.to_owned().into())
        );
    }
}
