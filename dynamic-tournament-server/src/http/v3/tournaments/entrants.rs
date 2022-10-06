use std::hash::{Hash, Hasher};

use dynamic_tournament_api::v3::id::{EntrantId, TournamentId};
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant, Player, Team};
use dynamic_tournament_api::Payload;
use dynamic_tournament_macros::{method, path};

use crate::http::etag::{Etag, HashEtag};
use crate::http::{Context, HttpResult, Response, Result};
use crate::{compare_etag, StatusCodeError};

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
    let entrants = ctx.state.store.entrants(id).list().await?;

    let etag = Etag::new(entrants.as_slice());
    compare_etag!(ctx, etag);

    Ok(Response::ok().etag(etag).json(&entrants))
}

async fn get(ctx: Context, tournament_id: TournamentId, id: EntrantId) -> Result {
    let entrant = ctx
        .state
        .store
        .get_entrant(tournament_id, id)
        .await
        .map_404()?;

    let etag = Etag::new(&entrant);
    compare_etag!(ctx, etag);

    Ok(Response::ok().etag(etag).json(&entrant))
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

    let entrant = ctx
        .state
        .store
        .get_entrant(tournament_id, id)
        .await
        .map_404()?;

    let etag = Etag::new(&entrant);
    compare_etag!(ctx, etag);

    ctx.state.store.entrants(tournament_id).delete(id).await?;

    Ok(Response::ok())
}

async fn patch(mut ctx: Context, tournament_id: TournamentId, id: EntrantId) -> Result {
    ctx.require_authentication()?;

    let entrant = ctx
        .state
        .store
        .get_entrant(tournament_id, id)
        .await
        .map_404()?;

    let etag = Etag::new(&entrant);
    compare_etag!(ctx, etag);

    let mut entrant = ctx.req.json().await?;
    ctx.state
        .store
        .entrants(tournament_id)
        .update(id, &entrant)
        .await?;

    entrant.id = id;

    Ok(Response::ok().json(&entrant))
}

impl HashEtag for [Entrant] {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        for elem in self {
            elem.hash(state);
        }
    }
}

impl HashEtag for Entrant {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let Self { id: _, inner } = self;

        match inner {
            EntrantVariant::Player(player) => {
                0u8.hash(state);
                player.hash(state);
            }
            EntrantVariant::Team(team) => {
                1u8.hash(state);
                team.hash(state);
            }
        }
    }
}

impl HashEtag for Player {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let Self { name, role, rating } = self;

        name.hash(state);
        role.hash(state);
        rating.hash(state);
    }
}

impl HashEtag for Team {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        let Self { name, players } = self;

        name.hash(state);
        for player in players {
            player.hash(state);
        }
    }
}
