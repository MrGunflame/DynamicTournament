use crate::Error;
use dynamic_tournament_api::v3::id::RoleId;
use dynamic_tournament_api::v3::tournaments::brackets::Bracket;
use dynamic_tournament_api::v3::tournaments::roles::Role;
use dynamic_tournament_api::v3::tournaments::PartialTournament;
use dynamic_tournament_api::v3::users::User;
use dynamic_tournament_api::v3::{
    id::{BracketId, EntrantId, TournamentId, UserId},
    tournaments::{entrants::Entrant, EntrantKind, Tournament, TournamentOverview},
};
use dynamic_tournament_core::{EntrantScore, Matches};
use sqlx::mysql::MySqlPool;
use sqlx::Row;

use futures::TryStreamExt;

#[derive(Clone, Debug)]
pub struct Store {
    pub pool: MySqlPool,
    pub table_prefix: String,
}

impl Store {
    #[inline]
    pub fn tournaments(&self) -> TournamentsClient<'_> {
        TournamentsClient { store: self }
    }

    #[inline]
    pub fn entrants(&self, id: TournamentId) -> EntrantsClient<'_> {
        EntrantsClient { store: self, id }
    }

    #[inline]
    pub fn roles(&self, id: TournamentId) -> RolesClient<'_> {
        RolesClient { store: self, id }
    }

    #[inline]
    pub fn users(&self) -> UsersClient<'_> {
        UsersClient { store: self }
    }

    pub async fn insert_tournament(&self, tournament: &Tournament) -> Result<TournamentId, Error> {
        let res = sqlx::query(&format!(
            "INSERT INTO {}tournaments (id, name, description, date, kind) VALUES (?, ?, ?, ?, ?)",
            self.table_prefix
        ))
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
        let sql = format!(
            "SELECT id, name, date, kind FROM {}tournaments",
            self.table_prefix
        );

        let mut rows = sqlx::query(&sql).fetch(&self.pool);

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
        let row = match sqlx::query(&format!(
            "SELECT name, date, description, kind FROM {}tournaments WHERE id = ?",
            self.table_prefix
        ))
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
        let res = sqlx::query(&format!(
            "INSERT INTO {}entrants (tournament_id, data) VALUES (?, ?)",
            self.table_prefix
        ))
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
        let row = match sqlx::query(&format!(
            "SELECT data FROM {}entrants WHERE tournament_id = ? AND id = ?",
            self.table_prefix
        ))
        .bind(tournament_id.0)
        .bind(id.0)
        .fetch_one(&self.pool)
        .await
        {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let mut entrant: Entrant = serde_json::from_slice(row.try_get("data")?)?;
        entrant.id = id;

        Ok(Some(entrant))
    }

    pub async fn get_entrants(&self, tournament_id: TournamentId) -> Result<Vec<Entrant>, Error> {
        let sql = format!(
            "SELECT id, data FROM {}entrants WHERE tournament_id = ?",
            self.table_prefix
        );

        let mut rows = sqlx::query(&sql).bind(tournament_id.0).fetch(&self.pool);

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
        let sql = format!(
            "SELECT id, data FROM {}brackets WHERE tournament_id = ?",
            self.table_prefix
        );

        let mut rows = sqlx::query(&sql).bind(tournament_id.0).fetch(&self.pool);

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
        let res = sqlx::query(&format!(
            "INSERT INTO {}brackets (tournament_id, data, state) VALUES (?, ?, ?)",
            self.table_prefix
        ))
        .bind(tournament_id.0)
        .bind(serde_json::to_vec(bracket)?)
        .bind(serde_json::to_vec::<Option<u8>>(&None)?)
        .execute(&self.pool)
        .await?;

        let id = res.last_insert_id();

        Ok(BracketId(id))
    }

    pub async fn get_bracket(
        &self,
        tournament_id: TournamentId,
        id: BracketId,
    ) -> Result<Option<Bracket>, Error> {
        let row = match sqlx::query(&format!(
            "SELECT data FROM {}brackets WHERE tournament_id = ? AND id = ?",
            self.table_prefix
        ))
        .bind(tournament_id.0)
        .bind(id.0)
        .fetch_one(&self.pool)
        .await
        {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        };

        let data: Vec<u8> = row.try_get("data")?;

        let mut bracket: Bracket = serde_json::from_slice(&data)?;
        bracket.id = id;

        Ok(Some(bracket))
    }

    pub async fn get_bracket_state(
        &self,
        tournament_id: TournamentId,
        id: BracketId,
    ) -> Result<Option<Matches<EntrantScore<u64>>>, Error> {
        let row = sqlx::query(&format!(
            "SELECT state FROM {}brackets WHERE tournament_id = ? AND id = ?",
            self.table_prefix
        ))
        .bind(tournament_id.0)
        .bind(id.0)
        .fetch_one(&self.pool)
        .await?;

        let state: Vec<u8> = row.try_get("state")?;

        let matches = serde_json::from_slice(&state)?;

        Ok(matches)
    }

    pub async fn update_bracket_state(
        &self,
        tournament_id: TournamentId,
        id: BracketId,
        state: &Option<Matches<EntrantScore<u64>>>,
    ) -> Result<(), Error> {
        sqlx::query(&format!(
            "UPDATE {}brackets SET state = ? WHERE tournament_id = ? AND id = ?",
            self.table_prefix
        ))
        .bind(serde_json::to_vec(state)?)
        .bind(tournament_id.0)
        .bind(id.0)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

macro_rules! get_one {
    ($query:expr) => {
        match $query {
            Ok(v) => v,
            Err(sqlx::Error::RowNotFound) => return Ok(None),
            Err(err) => return Err(err.into()),
        }
    };
}

#[derive(Copy, Clone, Debug)]
pub struct TournamentsClient<'a> {
    store: &'a Store,
}

impl<'a> TournamentsClient<'a> {
    /// Returns a list of all [`TournamentOverview`]s.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if an database error occured.
    pub async fn list(&self) -> Result<Vec<TournamentOverview>, Error> {
        let sql = format!(
            "SELECT id, name, date, kind FROM {}tournaments",
            self.store.table_prefix
        );

        let mut rows = sqlx::query(&sql).fetch(&self.store.pool);

        let mut tournaments = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let name = row.try_get("name")?;
            let date = row.try_get("date")?;
            let kind = row.try_get("kind")?;

            let id = TournamentId(id);
            let kind = EntrantKind::from_u8(kind).unwrap();

            tournaments.push(TournamentOverview {
                id,
                name,
                date,
                kind,
            });
        }

        Ok(tournaments)
    }

    /// Returns the [`Tournament`] with the given `id`. Returns `None` if no tournament with the
    /// given `id` exists.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if an database error occured.
    pub async fn get(&self, id: TournamentId) -> Result<Option<Tournament>, Error> {
        let row = get_one!(
            sqlx::query(&format!(
                "SELECT name, date, description, kind FROM {}tournaments WHERE id = ?",
                self.store.table_prefix
            ))
            .bind(id.0)
            .fetch_one(&self.store.pool)
            .await
        );

        let name = row.try_get("name")?;
        let description = row.try_get("description")?;
        let date = row.try_get("date")?;
        let kind = EntrantKind::from_u8(row.try_get("kind")?).unwrap();

        Ok(Some(Tournament {
            id,
            name,
            description,
            date,
            kind,
        }))
    }

    /// Inserts a new [`Tournament`] and returns the [`TournamentId`] for the newly created value.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if an database error occured.
    pub async fn insert(&self, tournament: &Tournament) -> Result<TournamentId, Error> {
        let res = sqlx::query(&format!(
            "INSERT INTO {}tournaments (name, description, date, kind) VALUES (?, ?, ?, ?)",
            self.store.table_prefix
        ))
        .bind(&tournament.name)
        .bind(&tournament.description)
        .bind(tournament.date)
        .bind(tournament.kind.to_u8())
        .execute(&self.store.pool)
        .await?;

        Ok(TournamentId(res.last_insert_id()))
    }

    /// Deletes the [`Tournament`] with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if an database error occured.
    pub async fn delete(&self, id: TournamentId) -> Result<(), Error> {
        // FIXME: Join all futures for better speeeed.
        sqlx::query(&format!(
            "DELETE FROM {}tournaments WHERE id = ?",
            self.store.table_prefix
        ))
        .bind(id.0)
        .execute(&self.store.pool)
        .await?;

        sqlx::query(&format!(
            "DELETE FROM {}entrants WHERE tournament_id = ?",
            self.store.table_prefix
        ))
        .bind(id.0)
        .execute(&self.store.pool)
        .await?;

        sqlx::query(&format!(
            "DELETE FROM {}brackets WHERE tournament_id = ?",
            self.store.table_prefix
        ))
        .bind(id.0)
        .execute(&self.store.pool)
        .await?;

        sqlx::query(&format!(
            "DELETE FROM {}roles WHERE tournament_id = ?",
            self.store.table_prefix
        ))
        .bind(id.0)
        .execute(&self.store.pool)
        .await?;

        Ok(())
    }

    /// Updates the [`Tournament`] with the given `id` using the given [`PartialTournament`].
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if an database error occured.
    ///
    /// # Panics
    ///
    /// Panics in debug mode if the tournament `kind` is changed without the entrants of that
    /// tournament being empty. This check is skipped in release mode.
    pub async fn update(
        &self,
        id: TournamentId,
        tournament: &PartialTournament,
    ) -> Result<(), Error> {
        if let Some(name) = &tournament.name {
            sqlx::query(&format!(
                "UPDATE {}tournaments SET name = ? WHERE id = ?",
                self.store.table_prefix
            ))
            .bind(name)
            .bind(id.0)
            .execute(&self.store.pool)
            .await?;
        }

        if let Some(description) = &tournament.description {
            sqlx::query(&format!(
                "UPDATE {}tournaments SET description = ? WHERE id = ?",
                self.store.table_prefix
            ))
            .bind(description)
            .bind(id.0)
            .execute(&self.store.pool)
            .await?;
        }

        if let Some(date) = tournament.date {
            sqlx::query(&format!(
                "UPDATE {}tournaments SET date = ? WHERE id = ?",
                self.store.table_prefix
            ))
            .bind(date)
            .bind(id.0)
            .execute(&self.store.pool)
            .await?;
        }

        if let Some(kind) = tournament.kind {
            #[cfg(debug_assertions)]
            {
                let entrants = self.store.entrants(id).list().await?;
                assert!(entrants.is_empty());
            }

            sqlx::query(&format!(
                "UPDATE {}tournaments SET kind = ? WHERE id = ?",
                self.store.table_prefix
            ))
            .bind(kind.to_u8())
            .bind(id.0)
            .execute(&self.store.pool)
            .await?;
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct EntrantsClient<'a> {
    store: &'a Store,
    id: TournamentId,
}

impl<'a> EntrantsClient<'a> {
    pub async fn list(&self) -> Result<Vec<Entrant>, Error> {
        let sql = format!(
            "SELECT id, data FROM {}entrants WHERE tournament_id = ?",
            self.store.table_prefix
        );

        let mut rows = sqlx::query(&sql).bind(self.id.0).fetch(&self.store.pool);

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

    pub async fn insert(&self, entrant: &Entrant) -> Result<EntrantId, Error> {
        let res = sqlx::query(&format!(
            "INSERT INTO {}entrants (tournament_id, data) VALUES (?, ?)",
            self.store.table_prefix
        ))
        .bind(self.id.0)
        .bind(serde_json::to_vec(entrant)?)
        .execute(&self.store.pool)
        .await?;

        Ok(EntrantId(res.last_insert_id()))
    }

    pub async fn delete(&self, id: EntrantId) -> Result<(), Error> {
        sqlx::query(&format!(
            "DELETE FROM {}entrants WHERE tournament_id = ? AND id = ?",
            self.store.table_prefix
        ))
        .bind(self.id.0)
        .bind(id.0)
        .execute(&self.store.pool)
        .await?;

        Ok(())
    }

    pub async fn update(&self, id: EntrantId, entrant: &Entrant) -> Result<(), Error> {
        sqlx::query(&format!(
            "UPDATE {}entrants SET data = ? WHERE id = ?",
            self.store.table_prefix,
        ))
        .bind(serde_json::to_vec(entrant)?)
        .bind(id.0)
        .execute(&self.store.pool)
        .await?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RolesClient<'a> {
    store: &'a Store,
    id: TournamentId,
}

impl<'a> RolesClient<'a> {
    pub async fn list(&self) -> Result<Vec<Role>, Error> {
        let sql = format!(
            "SELECT id, name FROM {}roles WHERE tournament_id = ?",
            self.store.table_prefix
        );

        let mut rows = sqlx::query(&sql).bind(self.id.0).fetch(&self.store.pool);

        let mut roles = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let id = row.try_get("id")?;
            let name = row.try_get("name")?;

            roles.push(Role {
                id: RoleId(id),
                name,
            });
        }

        Ok(roles)
    }

    pub async fn get(&self, id: RoleId) -> Result<Option<Role>, Error> {
        let row = get_one!(
            sqlx::query(&format!(
                "SELECT name FROM {}roles WHERE tournament_id = ? AND id = ?",
                self.store.table_prefix
            ))
            .bind(self.id.0)
            .bind(id.0)
            .fetch_one(&self.store.pool)
            .await
        );

        let name = row.try_get("name")?;

        Ok(Some(Role { id, name }))
    }

    pub async fn insert(&self, role: &Role) -> Result<RoleId, Error> {
        let res = sqlx::query(&format!(
            "INSERT INTO {}roles (name, tournament_id) VALUES (?, ?)",
            self.store.table_prefix
        ))
        .bind(&role.name)
        .bind(&self.id.0)
        .execute(&self.store.pool)
        .await?;

        Ok(RoleId(res.last_insert_id()))
    }

    pub async fn delete(&self, id: RoleId) -> Result<(), Error> {
        sqlx::query(&format!(
            "DELETE FROM {}roles WHERE id = ?",
            self.store.table_prefix
        ))
        .bind(id.0)
        .execute(&self.store.pool)
        .await?;

        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct UsersClient<'a> {
    store: &'a Store,
}

impl<'a> UsersClient<'a> {
    pub async fn get(&self, username: &str) -> Result<Option<User>, Error> {
        let row = get_one!(
            sqlx::query(&format!(
                "SELECT id, password FROM {}users WHERE name = ?",
                self.store.table_prefix
            ))
            .bind(username)
            .fetch_one(&self.store.pool)
            .await
        );

        let id = row.try_get("id")?;
        let password = row.try_get("password")?;

        Ok(Some(User {
            id: UserId(id),
            username: username.to_string(),
            password,
        }))
    }

    pub async fn insert(&self, user: &User) -> Result<(), Error> {
        sqlx::query(&format!(
            "INSERT INTO {}users (id, name, password) VALUES (?, ?, ?)",
            self.store.table_prefix
        ))
        .bind(user.id.0)
        .bind(&user.username)
        .bind(&user.password)
        .execute(&self.store.pool)
        .await?;

        Ok(())
    }
}
