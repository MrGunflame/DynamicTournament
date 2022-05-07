mod http;
mod logger;
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

use thiserror::Error;

use dynamic_tournament_api::tournament::{Tournament, TournamentId};

use futures::TryStreamExt;
use sqlx::Row;
use websocket::LiveBracket;

use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = read_config("config.toml");
    logger::init(config.loglevel);

    log::info!("Using config: {:?}", config);

    let users = read_users("users.json");

    let store = MySqlPool::connect(&config.database.connect_string()).await?;

    let state = State {
        store,
        users,
        subscribers: Arc::new(RwLock::new(HashMap::new())),
    };

    let tables = [
        "CREATE TABLE IF NOT EXISTS tournaments (id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY, data BLOB NOT NULL)",
        "CREATE TABLE IF NOT EXISTS tournaments_brackets (tournament_id BIGINT UNSIGNED PRIMARY KEY, data BLOB NOT NULL)"
    ];

    for t in tables {
        sqlx::query(t).execute(&state.store).await?;
    }

    http::bind(config.bind, state).await.unwrap();

    Ok(())
}

#[derive(Clone, Debug)]
pub struct State {
    store: MySqlPool,
    users: Vec<LoginData>,
    pub subscribers: Arc<RwLock<HashMap<u64, LiveBracket>>>,
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

    pub fn length_required() -> Self {
        Self::new(StatusCode::LENGTH_REQUIRED, "Length Required")
    }

    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST, "Bad Request")
    }

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

    pub fn decode_token(&self, token: &String) -> Result<Claims, jsonwebtoken::errors::Error> {
        let key = DecodingKey::from_secret(http::v2::auth::SECRET);
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);

        let data = jsonwebtoken::decode(token, &key, &validation)?;

        Ok(data.claims)
    }

    pub async fn list_tournaments(&self) -> Result<Vec<TournamentOverview>, Error> {
        let mut rows = sqlx::query("SELECT id, data FROM tournaments").fetch(&self.store);

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
            .fetch_one(&self.store)
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
            .execute(&self.store)
            .await?;

        let id = res.last_insert_id();

        Ok(id)
    }

    pub async fn update_bracket(&self, tournament_id: u64, bracket: Bracket) -> Result<(), Error> {
        let data = serde_json::to_vec(&bracket)?;

        sqlx::query("INSERT INTO tournaments_brackets (tournament_id, data) VALUES (?, ?) ON DUPLICATE KEY UPDATE data=VALUES(data)")
            .bind(tournament_id)
            .bind(data)
            .execute(&self.store)
            .await?;

        Ok(())
    }

    pub async fn get_bracket(&self, tournament_id: u64) -> Result<Option<Bracket>, Error> {
        let row = match sqlx::query("SELECT data FROM tournaments_brackets WHERE tournament_id = ?")
            .bind(tournament_id)
            .fetch_one(&self.store)
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

pub fn read_config<P>(path: P) -> Config
where
    P: AsRef<std::path::Path>,
{
    let mut file = std::fs::File::open(path).unwrap();

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    let mut config: Config = toml::from_slice(&buf).unwrap();

    if let Ok(val) = std::env::var("DYNT_LOGLEVEL") {
        let val = LevelFilter::from_str(&val).unwrap();

        config.loglevel = val;
    }

    if let Ok(val) = std::env::var("DYNT_BIND") {
        let val = SocketAddr::from_str(&val).unwrap();

        config.bind = val;
    }

    if let Ok(val) = std::env::var("DYNT_DB_DRIVER") {
        config.database.driver = val;
    }

    if let Ok(val) = std::env::var("DYNT_DB_HOST") {
        config.database.host = val;
    }

    if let Ok(val) = std::env::var("DYNT_DB_PORT") {
        let val = u16::from_str(&val).unwrap();

        config.database.port = val;
    }

    if let Ok(val) = std::env::var("DYNT_DB_USER") {
        config.database.user = val;
    }

    if let Ok(val) = std::env::var("DYNT_DB_PASSWORD") {
        config.database.password = val;
    }

    if let Ok(val) = std::env::var("DYNT_DB_DATABASE") {
        config.database.database = val;
    }

    config
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
pub struct Config {
    pub database: Database,
    pub loglevel: LevelFilter,
    pub bind: SocketAddr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}
