use crate::Error;
use dynamic_tournament_api::v3::{
    id::TournamentId,
    tournaments::{EntrantKind, Tournament, TournamentOverview},
};
use sqlx::Row;
use sqlx::{mysql::MySqlPool, Executor};

use futures::TryStreamExt;

#[derive(Clone, Debug)]
pub struct Store {
    pool: MySqlPool,
}

impl Store {
    pub async fn insert_tournament(&self, tournament: &Tournament) -> Result<TournamentId, Error> {
        let res = sqlx::query(
            "INSERT INTO tournaments (id, name, description, date, kind) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(tournament.id.as_ref())
        .bind(&tournament.name)
        .bind(&tournament.description)
        .bind(tournament.date)
        .bind(tournament.kind.to_u8())
        .execute(&self.pool)
        .await?;

        let id = res.last_insert_id();

        Ok(TournamentId(id))
    }

    pub async fn list_tournaments(&self) -> Result<Vec<TournamentOverview>, Error> {
        let mut rows =
            sqlx::query("SELECT id, name, date, kind FROM tournaments").fetch(&self.pool);

        let mut tournaments = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let name = row.try_get("name")?;
            let date = row.try_get("date")?;
            let kind = row.try_get("kind")?;

            tournaments.push(TournamentOverview {
                id: TournamentId(id),
                name,
                date,
                kind: EntrantKind::from_u8(kind).unwrap(),
            });
        }

        Ok(tournaments)
    }

    pub async fn get_tournament(&self, id: TournamentId) -> Result<Option<Tournament>, Error> {
        let row =
            match sqlx::query("SELECT name, date, description, kind FROM tournaments WHERE id = ?")
                .bind(id.as_ref())
                .fetch_one(&self.pool)
                .await
            {
                Ok(v) => v,
                Err(sqlx::Error::RowNotFound) => return Ok(None),
                Err(err) => return Err(err.into()),
            };

        Ok(Some(Tournament {
            id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            date: row.try_get("date")?,
            kind: EntrantKind::from_u8(row.try_get("kind")?).unwrap(),
            brackets: Vec::new(),
        }))
    }
}
