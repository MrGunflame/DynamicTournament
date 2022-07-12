mod matches;

use dynamic_tournament_api::v3::{
    id::{BracketId, SystemId, TournamentId},
    tournaments::brackets::Bracket,
};
use dynamic_tournament_core::{options::TournamentOptions, EntrantScore, SingleElimination};
use hyper::Method;

use crate::method;
use crate::{
    http::{Request, RequestUri, Response, Result},
    StatusCodeError,
};

pub async fn route(req: Request, mut uri: RequestUri<'_>, tournament_id: TournamentId) -> Result {
    match uri.take() {
        None => method!(req, {
            Method::GET => list(req, tournament_id).await,
            Method::POST => create(req, tournament_id).await,
        }),
        Some(part) => {
            let id = part.parse()?;

            match uri.take_str() {
                None => method!(req, {
                    Method::GET => get(req, tournament_id, id).await,
                }),
                Some("matches") => matches::route(req, uri, tournament_id, id).await,
                Some(_) => Err(StatusCodeError::not_found().into()),
            }
        }
    }
}

async fn list(req: Request, id: TournamentId) -> Result {
    let brackets = req.state().store.list_brackets(id).await?;

    Ok(Response::ok().json(&brackets))
}

async fn get(req: Request, tournament_id: TournamentId, id: BracketId) -> Result {
    let bracket = req.state().store.get_bracket(tournament_id, id).await?;

    match bracket {
        Some(bracket) => Ok(Response::ok().json(&bracket)),
        None => Err(StatusCodeError::not_found().into()),
    }
}

async fn create(mut req: Request, tournament_id: TournamentId) -> Result {
    req.require_authentication()?;

    let mut bracket: Bracket = req.json().await?;

    // Make sure all entrants in the bracket actually exist.
    let entrants = req.state().store.get_entrants(tournament_id).await?;

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

    bracket.options = match bracket.options.merge(options) {
        Ok(v) => v,
        Err(err) => {
            return Err(StatusCodeError::bad_request().message(err).into());
        }
    };

    let id = req
        .state()
        .store
        .insert_bracket(tournament_id, &bracket)
        .await?;
    bracket.id = id;

    Ok(Response::created().json(&bracket))
}
