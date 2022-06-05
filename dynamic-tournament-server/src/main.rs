mod config;
mod http;
mod logger;
mod store;
mod websocket;

use dynamic_tournament_api::auth::Claims;
use dynamic_tournament_api::tournament::{Bracket, TournamentOverview};
use hyper::StatusCode;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Validation;
use log::LevelFilter;
use parking_lot::RwLock;
use serde::Deserialize;
use serde::Serialize;
use sqlx::mysql::MySqlPool;

use store::Store;
use thiserror::Error;

use dynamic_tournament_api::tournament::{Tournament, TournamentId};

use futures::TryStreamExt;
use sqlx::Row;
use tokio::sync::{mpsc, watch};
use websocket::LiveBracket;

use std::collections::HashMap;
use std::io::Read;
use std::sync::Arc;

use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(short, long, value_name = "FILE", default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    logger::init(LevelFilter::Error);

    let args = Args::parse();

    let config = match config::Config::from_file(&args.config).await {
        Ok(config) => config.with_environment(),
        Err(file_err) => match config::Config::from_environment() {
            Ok(config) => config,
            Err(env_err) => {
                log::error!("Failed to load configuration, exiting");
                log::error!("Failed to load config file: {}", file_err);
                log::error!("Failed to load config from environment: {}", env_err);
                return Ok(());
            }
        },
    };

    logger::init(config.loglevel);

    log::info!("Using config: {:?}", config);

    let users = read_users("users.json");

    let store = MySqlPool::connect(&config.database.connect_string()).await?;

    let (shutdown_responder_tx, mut shutdown_responder_rx) = mpsc::channel(1);
    let (shutdown_tx, shutdown_rx) = watch::channel(None);

    tokio::task::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        log::info!("Interrupt");

        let _ = shutdown_tx.send(Some(shutdown_responder_tx));
    });

    let store = Store { pool: store };

    let state = State {
        store,
        users,
        subscribers: Arc::new(RwLock::new(HashMap::new())),
        shutdown_rx,
    };

    let tables = [
        "CREATE TABLE IF NOT EXISTS tournaments (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            date TIMESTAMP NOT NULL,
            kind TINYINT UNSIGNED NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS entrants (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            tournament_id BIGINT UNSIGNED NOT NULL,
            data BLOB NOT NULL
        )",
        "CREATE TABLE IF NOT EXISTS brackets (
            id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
            tournament_id BIGINT UNSIGNED NOT NULL,
            data BLOB NOT NULL
        )",
    ];

    for t in tables {
        sqlx::query(t).execute(&state.store.pool).await?;
    }

    http::bind(config.bind, state).await.unwrap();

    // Wait for all shutdown listeners to complete.
    while (shutdown_responder_rx.recv().await).is_some() {}

    Ok(())
}

#[derive(Clone, Debug)]
pub struct State {
    pub store: Store,
    users: Vec<LoginData>,
    pub subscribers: Arc<RwLock<HashMap<u64, LiveBracket>>>,
    // Note: Clone before polling.
    pub shutdown_rx: watch::Receiver<Option<mpsc::Sender<bool>>>,
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
    Bracket(#[from] dynamic_tournament_generator::Error),
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

    /// 411 Length Required
    pub fn length_required() -> Self {
        Self::new(StatusCode::LENGTH_REQUIRED, "Length Required")
    }

    /// 413 Payload Too Large
    pub fn payload_too_large() -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, "Payload Too Large")
    }
}

impl State {
    pub fn is_allowed(&self, data: &LoginData) -> bool {
        log::debug!("Trying to authenticate: {:?}", data);

        for user in &self.users {
            if user.username == data.username && user.password == data.password {
                return true;
            }
        }

        false
    }

    pub fn is_authenticated(&self, req: &http::Request) -> bool {
        let header = match req.headers().get("Authorization") {
            Some(header) => header.as_bytes(),
            None => return false,
        };

        let header = match header.as_ref().strip_prefix(b"Bearer ") {
            Some(header) => header,
            None => return false,
        };

        self.is_authenticated_string(header)
    }

    pub fn is_authenticated_string(&self, header: impl AsRef<[u8]>) -> bool {
        match String::from_utf8(header.as_ref().to_vec()) {
            Ok(s) => self.decode_token(&s).is_ok(),
            Err(err) => {
                log::info!("Failed to convert header to string: {:?}", err);
                false
            }
        }
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let key = DecodingKey::from_secret(http::v2::auth::SECRET);
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        let data = jsonwebtoken::decode(token, &key, &validation)?;

        Ok(data.claims)
    }

    pub async fn list_tournaments(&self) -> Result<Vec<TournamentOverview>, Error> {
        let mut rows = sqlx::query("SELECT id, data FROM tournaments").fetch(&self.store.pool);

        let mut tournaments = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let data: Vec<u8> = row.try_get("data")?;

            let data: Tournament = serde_json::from_slice(&data).unwrap();

            tournaments.push(TournamentOverview {
                id: TournamentId(id),
                name: data.name,
                date: data.date,
                bracket_type: data.bracket_type,
                entrants: data.entrants.len().try_into().unwrap(),
            });
        }

        Ok(tournaments)
    }

    pub async fn get_tournament(&self, id: u64) -> Result<Option<Tournament>, Error> {
        let row = match sqlx::query("SELECT data FROM tournaments WHERE id = ?")
            .bind(id)
            .fetch_one(&self.store.pool)
            .await
        {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let data: Vec<u8> = row.try_get("data")?;
        let mut data: Tournament = serde_json::from_slice(&data).unwrap();
        data.id = TournamentId(id);

        Ok(Some(data))
    }

    pub async fn create_tournament(&self, tournament: Tournament) -> Result<u64, Error> {
        let res = sqlx::query("INSERT INTO tournaments (data) VALUES (?)")
            .bind(serde_json::to_vec(&tournament).unwrap())
            .execute(&self.store.pool)
            .await?;

        let id = res.last_insert_id();

        Ok(id)
    }

    pub async fn update_bracket(&self, tournament_id: u64, bracket: Bracket) -> Result<(), Error> {
        let data = serde_json::to_vec(&bracket)?;

        sqlx::query("INSERT INTO tournaments_brackets (tournament_id, data) VALUES (?, ?) ON DUPLICATE KEY UPDATE data=VALUES(data)")
            .bind(tournament_id)
            .bind(data)
            .execute(&self.store.pool)
            .await?;

        Ok(())
    }

    pub async fn get_bracket(&self, tournament_id: u64) -> Result<Option<Bracket>, Error> {
        let row = match sqlx::query("SELECT data FROM tournaments_brackets WHERE tournament_id = ?")
            .bind(tournament_id)
            .fetch_one(&self.store.pool)
            .await
        {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let slice = row.try_get("data")?;

        let bracket = serde_json::from_slice(slice)?;

        Ok(Some(bracket))
    }
}

pub fn read_users<P>(path: P) -> Vec<LoginData>
where
    P: AsRef<std::path::Path>,
{
    let mut file = std::fs::File::open(path).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    serde_json::from_slice(&buf).unwrap()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}
