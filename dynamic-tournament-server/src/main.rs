mod auth;
mod config;
mod http;
mod logger;
mod signal;
mod state;
mod store;
mod websocket;

#[cfg(feature = "metrics")]
mod metrics;

use config::Config;

use crate::{signal::ShutdownListener, state::State};
use clap::Parser;
use hyper::StatusCode;
use thiserror::Error;

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(short, long, value_name = "FILE", default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let config = match config::Config::from_file(&args.config).await {
        Ok(config) => config.with_environment(),
        Err(file_err) => match config::Config::from_environment() {
            Ok(config) => config,
            Err(env_err) => {
                println!("Failed to load configuration, exiting");
                println!("Failed to load config file: {}", file_err);
                println!("Failed to load config from environment: {}", env_err);
                return Ok(());
            }
        },
    };

    logger::init(config.log.clone());

    log::info!("Using config: {:?}", config);

    let prefix = config.database.prefix.clone();
    let state = State::new(config);

    {
        let state = state.clone();

        tokio::task::spawn(async move {
            let tables = [
                format!(
                    "CREATE TABLE IF NOT EXISTS {}tournaments (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            date TIMESTAMP NOT NULL,
            kind TINYINT UNSIGNED NOT NULL
        )",
                    prefix
                ),
                format!(
                    "CREATE TABLE IF NOT EXISTS {}entrants (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            tournament_id BIGINT UNSIGNED NOT NULL,
            data BLOB NOT NULL
        )",
                    prefix
                ),
                format!(
                    "CREATE TABLE IF NOT EXISTS {}brackets (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            tournament_id BIGINT UNSIGNED NOT NULL,
            data BLOB NOT NULL,
            state BLOB NOT NULL
        )",
                    prefix
                ),
                format!(
                    "CREATE TABLE IF NOT EXISTS {}roles (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            tournament_id BIGINT UNSIGNED NOT NULL,
            name TEXT NOT NULL
        )",
                    prefix
                ),
                format!(
                    "CREATE TABLE IF NOT EXISTS {}users (
                        id BIGINT UNSIGNED PRIMARY KEY,
                        name TEXT NOT NULL,
                        password TEXT NOT NULL
                        )",
                    prefix
                ),
            ];

            for t in tables {
                if let Err(err) = sqlx::query(&t).execute(&state.store.pool).await {
                    log::error!("Failed to create tables: {}", err);
                    break;
                }
            }

            match state.store.users().get("admin").await {
                Ok(Some(_)) => log::debug!("Admin user already exists"),
                Ok(None) => {
                    log::debug!("Creating admin user");

                    // Generate admin user if it doesn't exist.
                    let user = auth::generate_admin_user();
                    match state.store.users().insert(&user).await {
                        Ok(()) => log::debug!("Admin user created"),
                        Err(err) => {
                            log::error!("Failed to create admin user: {}", err);
                        }
                    }
                }
                Err(err) => log::error!("Failed to create admin user: {}", err),
            }
        });
    }

    tokio::task::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        log::info!("Interrupt");
        signal::terminate().await;
    });

    loop {
        let shutdown = ShutdownListener::new();
        tokio::select! {
            res = http::bind(state.config.bind.clone(), state.clone()) => {
                match res {
                    Ok(()) => break,
                    Err(err) => log::error!("Failed to bind server: {}", err),
                }
            }
            _ = shutdown => {
                break;
            }
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Store(#[from] sqlx::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Hyper(#[from] hyper::Error),
    #[error("not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("bad request")]
    BadRequest,
    #[error("status code error")]
    StatusCodeError(#[from] StatusCodeError),
    #[error("{0}")]
    JsonWebToken(#[from] jsonwebtoken::errors::Error),
    #[error("{0}")]
    Bracket(#[from] dynamic_tournament_core::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("body already consumed")]
    BodyConsumed,
    #[error("invalid token")]
    InvalidToken,
}

#[derive(Debug, Error)]
#[error("error {code}: {message}")]
pub struct StatusCodeError {
    code: StatusCode,
    message: String,
}

impl StatusCodeError {
    pub fn new<T>(code: StatusCode, message: T) -> Self
    where
        T: ToString,
    {
        Self {
            code,
            message: message.to_string(),
        }
    }

    /// 400 Bad Request
    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST, "Bad Request")
    }

    /// 401 Unauthorized
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "Unauthorized")
    }

    /// 403 Forbidden
    pub fn forbidden() -> Self {
        Self::new(StatusCode::FORBIDDEN, "Forbidden")
    }

    /// 404 Not Found
    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "Not Found")
    }

    /// 405 Method Not Allowed
    pub fn method_not_allowed() -> Self {
        Self::new(StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed")
    }

    /// 408 Request Timeout
    pub fn request_timeout() -> Self {
        Self::new(StatusCode::REQUEST_TIMEOUT, "Request Timeout")
    }

    /// 410 Gone
    pub fn gone() -> Self {
        Self::new(StatusCode::GONE, "Gone")
    }

    /// 411 Length Required
    pub fn length_required() -> Self {
        Self::new(StatusCode::LENGTH_REQUIRED, "Length Required")
    }

    /// 413 Payload Too Large
    pub fn payload_too_large() -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, "Payload Too Large")
    }

    /// 426 Upgrade Required
    pub fn upgrade_required() -> Self {
        Self::new(StatusCode::UPGRADE_REQUIRED, "Upgrade Required")
    }

    /// Sets the message of the error.
    pub fn message<T>(mut self, msg: T) -> Self
    where
        T: ToString,
    {
        self.message = msg.to_string();
        self
    }
}
