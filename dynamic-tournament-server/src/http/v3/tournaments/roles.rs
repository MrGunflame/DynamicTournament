use dynamic_tournament_api::auth::Flags;
use dynamic_tournament_api::v3::id::{RoleId, TournamentId};
use dynamic_tournament_api::v3::tournaments::roles::Role;
use dynamic_tournament_api::Payload;
use dynamic_tournament_macros::{method, path};

use crate::http::{Context, Response, Result};
use crate::StatusCodeError;

pub async fn route(mut ctx: Context, tournament_id: TournamentId) -> Result {
    path!(ctx, {
        @ => method!(ctx, {
            GET => list(ctx, tournament_id).await,
            POST => create(ctx, tournament_id).await,
        }),
        id => method!(ctx, {
            GET => get(ctx, tournament_id, id).await,
            DELETE => delete(ctx, tournament_id, id).await,
        })
    })
}

async fn list(ctx: Context, id: TournamentId) -> Result {
    let roles = ctx.state.store.roles(id).list().await?;

    Ok(Response::ok().json(&roles))
}

async fn get(ctx: Context, tournament_id: TournamentId, id: RoleId) -> Result {
    let role = ctx.state.store.roles(tournament_id).get(id).await?;

    match role {
        Some(role) => Ok(Response::ok().json(&role)),
        None => Err(StatusCodeError::not_found().into()),
    }
}

async fn create(mut ctx: Context, tournament_id: TournamentId) -> Result {
    ctx.require_authentication(Flags::ADMIN)?;

    let mut roles: Payload<Role> = ctx.req.json().await?;

    for role in roles.iter_mut() {
        role.id = ctx.state.store.roles(tournament_id).insert(role).await?;
    }

    Ok(Response::created().json(&roles))
}

async fn delete(ctx: Context, tournament_id: TournamentId, id: RoleId) -> Result {
    ctx.require_authentication(Flags::ADMIN)?;

    ctx.state.store.roles(tournament_id).delete(id).await?;

    Ok(Response::ok())
}
