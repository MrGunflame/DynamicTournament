mod http;
mod logger;
mod websocket;

use chrono::DateTime;
use chrono::Utc;
use dynamic_tournament_api::tournament::{Bracket, TournamentOverview};
use log::LevelFilter;
use parking_lot::RwLock;
use serde::Deserialize;
use serde::Serialize;
use sqlx::mysql::MySqlPool;

use thiserror::Error;

use dynamic_tournament_api::tournament::Player;
use dynamic_tournament_api::tournament::Team;
use dynamic_tournament_api::tournament::{Tournament, TournamentId};

use futures::TryStreamExt;
use sqlx::Row;
use tokio::sync::broadcast;
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
        "CREATE TABLE IF NOT EXISTS tournaments (id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY, name TEXT NOT NULL, date TEXT NOT NULL, description TEXT NOT NULL, bracket_type TINYINT UNSIGNED NOT NULL)",
        "CREATE TABLE IF NOT EXISTS tournaments_teams (tournament_id BIGINT UNSIGNED NOT NULL, name TEXT NOT NULL, team_index BIGINT UNSIGNED NOT NULL)",
        "CREATE TABLE IF NOT EXISTS tournaments_teams_players (tournament_id BIGINT UNSIGNED NOT NULL, account_name TEXT NOT NULL, role TINYINT UNSIGNED, team_index BIGINT UNSIGNED NOT NULL)",
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

    pub fn is_authenticated<T>(&self, req: &hyper::Request<T>) -> bool {
        let header = match req.headers().get("Authorization") {
            Some(header) => header.as_bytes(),
            None => return false,
        };

        self.is_authenticated_string(header)
    }

    pub fn is_authenticated_string(&self, header: impl AsRef<[u8]>) -> bool {
        let header = match header.as_ref().strip_prefix(b"Basic ") {
            Some(header) => header,
            None => return false,
        };

        let data = match base64::decode(header) {
            Ok(v) => v,
            Err(err) => {
                log::debug!("Authorization header decoding failed: {:?}", err);

                return false;
            }
        };

        let string = match std::str::from_utf8(&data) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let (username, password) = match string.split_once(":") {
            Some((u, p)) => (u, p),
            None => return false,
        };

        let data = LoginData {
            username: username.to_owned(),
            password: password.to_owned(),
        };

        self.is_allowed(&data)
    }

    pub async fn list_tournaments(&self) -> Result<Vec<TournamentOverview>, Error> {
        let mut rows =
            sqlx::query("SELECT id, name, date, bracket_type FROM tournaments").fetch(&self.store);

        let mut tournaments = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let name = row.try_get("name")?;
            let date: String = row.try_get("date")?;
            let bracket_type: u8 = row.try_get("bracket_type")?;

            let row = sqlx::query(
                "SELECT COUNT(*) AS teams FROM tournaments_teams WHERE tournament_id = ?",
            )
            .bind(id)
            .fetch_one(&self.store)
            .await?;

            let teams: i64 = row.try_get("teams")?;

            tournaments.push(TournamentOverview {
                id: TournamentId(id),
                name,
                date: DateTime::parse_from_rfc3339(&date)
                    .unwrap()
                    .with_timezone(&Utc),
                bracket_type: bracket_type.try_into().unwrap(),
                teams: teams as u64,
            });
        }

        Ok(tournaments)
    }

    pub async fn get_tournament(&self, id: u64) -> Result<Option<Tournament>, Error> {
        let row = match sqlx::query(
            "SELECT name, date, bracket_type, description FROM tournaments WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.store)
        .await
        {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let mut tournament = Tournament {
            id: TournamentId(id),
            name: row.try_get("name")?,
            date: DateTime::parse_from_rfc3339(&row.try_get::<'_, String, _>("date")?)
                .unwrap()
                .with_timezone(&Utc),
            bracket_type: row
                .try_get::<'_, u8, _>("bracket_type")?
                .try_into()
                .unwrap(),
            description: row.try_get("description")?,
            teams: Vec::new(),
        };

        let mut rows = sqlx::query(
            "SELECT name FROM tournaments_teams WHERE tournament_id = ? ORDER BY team_index ASC",
        )
        .bind(tournament.id.0)
        .fetch(&self.store);

        while let Some(row) = rows.try_next().await? {
            let team = Team {
                name: row.try_get("name")?,
                players: Vec::new(),
            };

            tournament.teams.push(team);
        }

        let mut rows = sqlx::query(
            "SELECT account_name, role, team_index FROM tournaments_teams_players WHERE tournament_id = ?",
                )
                .bind(tournament.id.0)
                .fetch(&self.store);

        while let Some(row) = rows.try_next().await? {
            let player = Player {
                account_name: row.try_get("account_name")?,
                role: row.try_get::<'_, u8, _>("role")?.try_into().unwrap(),
            };

            let team_index: u64 = row.try_get("team_index")?;

            tournament.teams[team_index as usize].players.push(player);
        }

        Ok(Some(tournament))
    }

    pub async fn create_tournament(&self, tournament: Tournament) -> Result<u64, Error> {
        let res = sqlx::query(
            "INSERT INTO tournaments (name, date, bracket_type, description) VALUES (?, ?, ?, ?)",
        )
        .bind(tournament.name)
        .bind(tournament.date.to_rfc3339())
        .bind(u8::from(tournament.bracket_type))
        .bind(tournament.description)
        .execute(&self.store)
        .await?;

        let id = res.last_insert_id();

        for (i, team) in tournament.teams.into_iter().enumerate() {
            sqlx::query(
                "INSERT INTO tournaments_teams (name, tournament_id, team_index) VALUES (?, ?, ?)",
            )
            .bind(team.name)
            .bind(id)
            .bind(i as u64)
            .execute(&self.store)
            .await?;

            for player in team.players {
                sqlx::query("INSERT INTO tournaments_teams_players (tournament_id, team_index, account_name, role) VALUES (?, ?, ?, ?)")
                .bind(id)
                .bind(i as u64)
                .bind(player.account_name)
                .bind(u8::from(player.role))
                .execute(&self.store).await?;
            }
        }

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
