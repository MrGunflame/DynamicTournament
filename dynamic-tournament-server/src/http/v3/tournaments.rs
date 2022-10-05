mod brackets;
mod entrants;
mod roles;

use std::hash::{Hash, Hasher};

use dynamic_tournament_api::v3::tournaments::Tournament;
use dynamic_tournament_api::v3::{id::TournamentId, tournaments::TournamentOverview};
use dynamic_tournament_api::Payload;
use dynamic_tournament_macros::{method, path};

use crate::http::etag::HashEtag;
use crate::{
    compare_etag,
    http::{etag::Etag, Context, Response, Result},
    StatusCodeError,
};

pub async fn route(mut ctx: Context) -> Result {
    path!(ctx, {
        @ => method!(ctx, {
            GET => list(ctx).await,
            POST => create(ctx).await,
        }),
        id => {
            // Check if the tournament exists before continuing.
            if ctx.state.store.tournaments().get(id).await?.is_none() {
                return Err(StatusCodeError::not_found()
                    .message("Invalid tournament id")
                    .into());
            }

            path!(ctx, {
                "entrants" => entrants::route(ctx, id).await,
                "brackets" => brackets::route(ctx, id).await,
                "roles" => roles::route(ctx, id).await,
                @ => method!(ctx, {
                    GET => get(ctx, id).await,
                    PATCH => patch(ctx, id).await,
                    DELETE => delete(ctx, id).await,
                }),
            })
        }
    })
}

async fn list(ctx: Context) -> Result {
    let tournaments = ctx.state.store.tournaments().list().await?;

    let etag = Etag::new(tournaments.as_slice());
    compare_etag!(ctx, etag);

    Ok(Response::ok().etag(etag).json(&tournaments))
}

async fn get(ctx: Context, id: TournamentId) -> Result {
    let tournament = ctx.state.store.tournaments().get(id).await?;

    let tournament = tournament.ok_or_else(StatusCodeError::not_found)?;

    let etag = Etag::new(&tournament);
    compare_etag!(ctx, etag);

    Ok(Response::ok().etag(etag).json(&tournament))
}

async fn create(mut ctx: Context) -> Result {
    ctx.require_authentication()?;

    let mut tournaments: Payload<Tournament> = ctx.req.json().await?;

    for tournament in tournaments.iter_mut() {
        tournament.id = ctx.state.store.tournaments().insert(tournament).await?;
    }

    Ok(Response::created().json(&tournaments))
}

async fn patch(mut ctx: Context, id: TournamentId) -> Result {
    ctx.require_authentication()?;

    // Check if the tournament exists.
    let mut tournament = match ctx.state.store.tournaments().get(id).await? {
        Some(tournament) => tournament,
        None => return Err(StatusCodeError::not_found().into()),
    };

    let partial = ctx.req.json().await?;
    ctx.state.store.tournaments().update(id, &partial).await?;

    // Merge the patch.
    tournament.update(partial);

    Ok(Response::ok().json(&tournament))
}

async fn delete(ctx: Context, id: TournamentId) -> Result {
    ctx.require_authentication()?;

    ctx.state.store.tournaments().delete(id).await?;

    Ok(Response::ok())
}

impl HashEtag for [TournamentOverview] {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        for elem in self {
            elem.id.hash(state);
            elem.name.hash(state);
            elem.date.hash(state);
            elem.kind.hash(state);
        }
    }
}

impl HashEtag for Tournament {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.name.hash(state);
        self.description.hash(state);
        self.date.hash(state);
        self.kind.hash(state);
    }
}
