use crate::Error;
use dynamic_tournament_api::v3::tournaments::brackets::Bracket;
use dynamic_tournament_api::v3::{
    id::{BracketId, EntrantId, TournamentId},
    tournaments::{
        entrants::{Entrant, EntrantVariant},
        EntrantKind, Tournament, TournamentOverview,
    },
};
use sqlx::Row;
use sqlx::{mysql::MySqlPool, Executor};

use futures::TryStreamExt;

#[derive(Clone, Debug)]
pub struct Store {
    pub pool: MySqlPool,
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
        }))
    }

    pub async fn insert_entrant(
        &self,
        tournament_id: TournamentId,
        entrant: Entrant,
    ) -> Result<EntrantId, Error> {
        let res = sqlx::query("INSERT INTO entrants (tournament_id, data) VALUES (?, ?)")
            .bind(tournament_id.0)
            .bind(serde_json::to_vec(&entrant)?)
            .execute(&self.pool)
            .await?;

        let id = res.last_insert_id();

        Ok(EntrantId(id))
    }

    pub async fn get_entrant(
        &self,
        tournament_id: TournamentId,
        id: EntrantId,
    ) -> Result<Option<Entrant>, Error> {
        let row = match sqlx::query("SELECT data FROM entrants WHERE tournament_id = ? AND id = ?")
            .bind(tournament_id.0)
            .bind(id.0)
            .fetch_one(&self.pool)
            .await
        {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let entrant = serde_json::from_slice(row.try_get("data")?)?;

        Ok(Some(entrant))
    }

    pub async fn get_entrants(&self, tournament_id: TournamentId) -> Result<Vec<Entrant>, Error> {
        let mut rows = sqlx::query("SELECT id, data FROM entrants WHERE tournament_id = ?")
            .bind(tournament_id.0)
            .fetch(&self.pool);

        let mut entrants = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let data: Vec<u8> = row.try_get("data")?;

            let mut inner: Entrant = serde_json::from_slice(&data)?;
            inner.id = EntrantId(id);

            entrants.push(inner);
        }

        Ok(entrants)
    }

    pub async fn list_brackets(&self, tournament_id: TournamentId) -> Result<Vec<Bracket>, Error> {
        let mut rows = sqlx::query("SELECT id, data FROM brackets WHERE tournament_id = ?")
            .bind(tournament_id.0)
            .fetch(&self.pool);

        let mut brackets = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let data: Vec<u8> = row.try_get("data")?;

            let mut bracket: Bracket = serde_json::from_slice(&data)?;
            bracket.id = BracketId(id);

            brackets.push(bracket);
        }

        Ok(brackets)
    }

    pub async fn insert_bracket(
        &self,
        tournament_id: TournamentId,
        bracket: &Bracket,
    ) -> Result<BracketId, Error> {
        let res = sqlx::query("INSERT INTO brackets (tournament_id, data) VALUES (?, ?)")
            .bind(tournament_id.0)
            .bind(serde_json::to_vec(bracket)?)
            .execute(&self.pool)
            .await?;

        let id = res.last_insert_id();

        Ok(BracketId(id))
    }
}
