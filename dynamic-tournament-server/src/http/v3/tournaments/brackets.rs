mod matches;

use dynamic_tournament_api::{
    v3::{
        id::{BracketId, SystemId, TournamentId},
        tournaments::brackets::Bracket,
    },
    Payload,
};
use dynamic_tournament_core::{options::TournamentOptions, EntrantScore, SingleElimination};
use dynamic_tournament_macros::{method, path};

use crate::{
    http::{Context, Response, Result},
    StatusCodeError,
};

pub async fn route(mut ctx: Context, tournament_id: TournamentId) -> Result {
    path!(ctx, {
        @ => method!(ctx, {
            GET => list(ctx, tournament_id).await,
            POST => create(ctx, tournament_id).await,
        }),
        id => path!(ctx, {
            @ => method!(ctx, {
                GET => get(ctx, tournament_id, id).await,
            }),
            "matches" => matches::route(ctx, tournament_id, id).await,
        })
    })
}

async fn list(ctx: Context, id: TournamentId) -> Result {
    let brackets = ctx.state.store.list_brackets(id).await?;

    Ok(Response::ok().json(&brackets))
}

async fn get(ctx: Context, tournament_id: TournamentId, id: BracketId) -> Result {
    let bracket = ctx.state.store.get_bracket(tournament_id, id).await?;

    match bracket {
        Some(bracket) => Ok(Response::ok().json(&bracket)),
        None => Err(StatusCodeError::not_found().into()),
    }
}

async fn create(mut ctx: Context, tournament_id: TournamentId) -> Result {
    ctx.require_authentication()?;

    let mut brackets: Payload<Bracket> = ctx.req.json().await?;

    for bracket in brackets.iter_mut() {
        // Make sure all entrants in the bracket actually exist.
        let entrants = ctx.state.store.get_entrants(tournament_id).await?;

        // Keep track of consumed ids to deny duplicates.
        let mut consumed = Vec::with_capacity(bracket.entrants.len());

        for id in bracket.entrants.iter() {
            if consumed.contains(id) {
                return Err(StatusCodeError::bad_request()
                    .message(format!("found entrant {} multiple times", id))
                    .into());
            }

            if !entrants.iter().any(|e| e.id == *id) {
                return Err(StatusCodeError::bad_request()
                    .message(format!(
                        "invalid entrant {}, does not exist for tournament",
                        id
                    ))
                    .into());
            }

            consumed.push(*id);
        }

        let options = match bracket.system {
            SystemId(1) => SingleElimination::<u8, EntrantScore<u8>>::options(),
            SystemId(2) => TournamentOptions::default(),
            _ => unreachable!(),
        };

        bracket.options = match bracket.options.clone().merge(options) {
            Ok(v) => v,
            Err(err) => {
                return Err(StatusCodeError::bad_request().message(err).into());
            }
        };

        bracket.id = ctx
            .state
            .store
            .insert_bracket(tournament_id, bracket)
            .await?;
    }

    Ok(Response::created().json(&brackets))
}
