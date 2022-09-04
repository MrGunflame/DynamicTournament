mod brackets;
mod entrants;
mod roles;

use dynamic_tournament_api::v3::id::TournamentId;
use dynamic_tournament_api::v3::tournaments::Tournament;
use dynamic_tournament_api::Payload;
use dynamic_tournament_macros::{method, path};

use crate::{
    http::{Request, RequestUri, Response, Result},
    StatusCodeError,
};

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    path!(uri, {
        @ => method!(req, {
            GET => list(req).await,
            POST => create(req).await,
        }),
        id => {
            // Check if the tournament exists before continuing.
            if req.state().store.tournaments().get(id).await?.is_none() {
                return Err(StatusCodeError::not_found()
                    .message("Invalid tournament id")
                    .into());
            }

            path!(uri, {
                "entrants" => entrants::route(req, uri, id).await,
                "brackets" => brackets::route(req, uri, id).await,
                "roles" => roles::route(req, uri, id).await,
                @ => method!(req, {
                    GET => get(req, id).await,
                    PATCH => patch(req, id).await,
                    DELETE => delete(req, id).await,
                }),
            })
        }
    })
}

async fn list(req: Request) -> Result {
    let tournaments = req.state().store.tournaments().list().await?;

    Ok(Response::ok().json(&tournaments))
}

async fn get(req: Request, id: TournamentId) -> Result {
    let tournament = req.state().store.tournaments().get(id).await?;

    let tournament = tournament.ok_or_else(StatusCodeError::not_found)?;

    Ok(Response::ok().json(&tournament))
}

async fn create(mut req: Request) -> Result {
    req.require_authentication()?;

    let mut tournaments: Payload<Tournament> = req.json().await?;

    for tournament in tournaments.iter_mut() {
        tournament.id = req.state().store.tournaments().insert(tournament).await?;
    }

    Ok(Response::created().json(&tournaments))
}

async fn patch(mut req: Request, id: TournamentId) -> Result {
    req.require_authentication()?;

    // Check if the tournament exists.
    let mut tournament = match req.state().store.tournaments().get(id).await? {
        Some(tournament) => tournament,
        None => return Err(StatusCodeError::not_found().into()),
    };

    let partial = req.json().await?;
    req.state().store.tournaments().update(id, &partial).await?;

    // Merge the patch.
    tournament.update(partial);

    Ok(Response::ok().json(&tournament))
}

async fn delete(req: Request, id: TournamentId) -> Result {
    req.require_authentication()?;

    req.state().store.tournaments().delete(id).await?;

    Ok(Response::ok())
}
