use std::env;
use std::net::SocketAddr;
use std::{io, path::Path};

use jsonwebtoken::Algorithm;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
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
    pub bind: SocketAddr,

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
            bind: SocketAddr::new([0, 0, 0, 0].into(), 3000),
            authorization: Authorization::default(),
        }
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
        );

        self
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Authorization {
    pub algorithm: Algorithm,
}

impl Authorization {
    pub fn from_environment() -> Result<Self, ConfigError> {
        let mut this = Self::default();

        from_environment!(this, "DT_AUTH_ALGORITHM", algorithm);

        Ok(this)
    }

    pub fn with_environment(mut self) -> Self {
        from_environment!(self, "DT_AUTH_ALGORITHM", algorithm);

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
