use dynamic_tournament_api::v3::id::{EntrantId, TournamentId};
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
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
            PATCH => patch(ctx, tournament_id, id).await,
            DELETE => delete(ctx, tournament_id, id).await,
        })
    })
}

async fn list(ctx: Context, id: TournamentId) -> Result {
    let entrants = ctx.state.store.get_entrants(id).await?;

    Ok(Response::ok().json(&entrants))
}

async fn get(ctx: Context, tournament_id: TournamentId, id: EntrantId) -> Result {
    let entrant = ctx.state.store.get_entrant(tournament_id, id).await?;

    match entrant {
        Some(entrant) => Ok(Response::ok().json(&entrant)),
        None => Err(StatusCodeError::not_found().into()),
    }
}

async fn create(mut ctx: Context, tournament_id: TournamentId) -> Result {
    ctx.require_authentication()?;

    let tournament = ctx
        .state
        .store
        .get_tournament(tournament_id)
        .await?
        .unwrap();

    let mut entrants: Payload<Entrant> = ctx.req.json().await?;

    // Fetch roles for all entrants.
    let roles = ctx.state.store.roles(tournament_id).list().await?;

    for entrant in entrants.iter_mut() {
        // Check that the entrant matches the tournament kind.
        if tournament.kind != entrant.kind() {
            return Err(StatusCodeError::bad_request()
                .message("invalid entrant kind for this tournament")
                .into());
        }

        // Check if the roles for all players exist.
        match &entrant.inner {
            EntrantVariant::Player(player) => {
                if !roles.iter().any(|role| player.role == role.id) {
                    return Err(StatusCodeError::bad_request()
                        .message(format!("invalid role {} for player", player.role))
                        .into());
                }
            }
            EntrantVariant::Team(team) => {
                for player in &team.players {
                    if !roles
                        .iter()
                        .any(|role| player.role == 0 || player.role == role.id)
                    {
                        return Err(StatusCodeError::bad_request()
                            .message(format!("invalid role {} for player", player.role))
                            .into());
                    }
                }
            }
        }

        // Insert the entrant.
        entrant.id = ctx
            .state
            .store
            .entrants(tournament_id)
            .insert(entrant)
            .await?;
    }

    Ok(Response::created().json(&entrants))
}

async fn delete(ctx: Context, tournament_id: TournamentId, id: EntrantId) -> Result {
    ctx.require_authentication()?;

    ctx.state.store.entrants(tournament_id).delete(id).await?;

    Ok(Response::ok())
}

async fn patch(mut ctx: Context, tournament_id: TournamentId, id: EntrantId) -> Result {
    ctx.require_authentication()?;

    let mut entrant = ctx.req.json().await?;
    ctx.state
        .store
        .entrants(tournament_id)
        .update(id, &entrant)
        .await?;

    entrant.id = id;

    Ok(Response::ok().json(&entrant))
}
