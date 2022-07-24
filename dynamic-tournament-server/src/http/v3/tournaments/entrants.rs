use dynamic_tournament_api::v3::id::{EntrantId, TournamentId};
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use dynamic_tournament_api::Payload;
use hyper::Method;

use crate::http::{Request, RequestUri, Response, Result};
use crate::{method, StatusCodeError};

pub async fn route(req: Request, mut uri: RequestUri<'_>, tournament_id: TournamentId) -> Result {
    match uri.take() {
        None => method!(req, {
            Method::GET => list(req, tournament_id).await,
            Method::POST => create(req, tournament_id).await,
        }),
        Some(part) => {
            let id = part.parse()?;

            method!(req, {
                Method::GET => get(req, tournament_id, id).await,
            })
        }
    }
}

async fn list(req: Request, id: TournamentId) -> Result {
    let entrants = req.state().store.get_entrants(id).await?;

    Ok(Response::ok().json(&entrants))
}

async fn get(req: Request, tournament_id: TournamentId, id: EntrantId) -> Result {
    let entrant = req.state().store.get_entrant(tournament_id, id).await?;

    match entrant {
        Some(entrant) => Ok(Response::ok().json(&entrant)),
        None => Err(StatusCodeError::not_found().into()),
    }
}

async fn create(mut req: Request, tournament_id: TournamentId) -> Result {
    req.require_authentication()?;

    let tournament = req
        .state()
        .store
        .get_tournament(tournament_id)
        .await?
        .unwrap();

    let mut entrants: Payload<Entrant> = req.json().await?;

    // Fetch roles for all entrants.
    let roles = req.state().store.roles(tournament_id).list().await?;

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
                    if !roles.iter().any(|role| player.role == role.id) {
                        return Err(StatusCodeError::bad_request()
                            .message(format!("invalid role {} for player", player.role))
                            .into());
                    }
                }
            }
        }

        // Insert the entrant.
        entrant.id = req
            .state()
            .store
            .entrants(tournament_id)
            .insert(entrant)
            .await?;
    }

    Ok(Response::created().json(&entrants))
}
