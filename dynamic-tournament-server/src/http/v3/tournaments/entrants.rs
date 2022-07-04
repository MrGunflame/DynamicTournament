use dynamic_tournament_api::v3::id::{EntrantId, TournamentId};
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use hyper::{Body, Method, Response, StatusCode};

use crate::method;
use crate::{
    http::{Request, RequestUri},
    Error,
};

pub async fn route(
    req: Request,
    mut uri: RequestUri<'_>,
    tournament_id: TournamentId,
) -> Result<Response<Body>, Error> {
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

async fn list(req: Request, id: TournamentId) -> Result<Response<Body>, Error> {
    let entrants = req.state().store.get_entrants(id).await?;

    let body = serde_json::to_vec(&entrants)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))
        .unwrap())
}

async fn get(
    req: Request,
    tournament_id: TournamentId,
    id: EntrantId,
) -> Result<Response<Body>, Error> {
    let entrant = req.state().store.get_entrant(tournament_id, id).await?;

    let mut resp = Response::new(Body::empty());
    match entrant {
        Some(entrant) => {
            *resp.status_mut() = StatusCode::OK;
            *resp.body_mut() = Body::from(serde_json::to_vec(&entrant)?);
        }
        None => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
            *resp.body_mut() = Body::from("invalid entrant id");
        }
    }

    Ok(resp)
}

async fn create(mut req: Request, tournament_id: TournamentId) -> Result<Response<Body>, Error> {
    let tournament = req
        .state()
        .store
        .get_tournament(tournament_id)
        .await?
        .unwrap();

    let mut body: Entrant = req.json().await?;

    let mut resp = Response::new(Body::empty());
    if tournament.kind != body.kind() {
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        *resp.body_mut() = Body::from("invalid entrant kind for this tournament");
        return Ok(resp);
    }

    // Check if the roles for all players exist.
    let roles = req.state().store.roles(tournament_id).list().await?;
    match &body.inner {
        EntrantVariant::Player(player) => {
            if !roles.iter().any(|role| player.role == role.id) {
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                *resp.body_mut() = Body::from(format!("invalid role {} for player", player.role));
                return Ok(resp);
            }
        }
        EntrantVariant::Team(team) => {
            for player in &team.players {
                if !roles.iter().any(|role| player.role == role.id) {
                    *resp.status_mut() = StatusCode::BAD_REQUEST;
                    *resp.body_mut() =
                        Body::from(format!("invalid role {} for player", player.role));
                    return Ok(resp);
                }
            }
        }
    }

    // Insert the entrant.
    body.id = req
        .state()
        .store
        .entrants(tournament_id)
        .insert(&body)
        .await?;

    *resp.status_mut() = StatusCode::OK;
    *resp.body_mut() = Body::from(serde_json::to_vec(&body)?);

    Ok(resp)
}
