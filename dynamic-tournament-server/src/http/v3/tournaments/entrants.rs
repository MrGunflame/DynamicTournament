use dynamic_tournament_api::v3::id::{EntrantId, TournamentId};
use dynamic_tournament_api::v3::tournaments::entrants::Entrant;
use hyper::{Body, Method, Response, StatusCode};

use crate::method;
use crate::{
    http::{Request, RequestUri},
    Error, State,
};

pub async fn route(
    req: Request,
    mut uri: RequestUri<'_>,
    state: State,
    tournament_id: TournamentId,
) -> Result<Response<Body>, Error> {
    match uri.take() {
        None => method!(req, {
            Method::GET => list(req, state, tournament_id).await,
            Method::POST => create(req, state, tournament_id).await,
        }),
        Some(part) => {
            let id = part.parse()?;

            method!(req, {
                Method::GET => get(req, state, tournament_id, id).await,
            })
        }
    }
}

async fn list(_req: Request, state: State, id: TournamentId) -> Result<Response<Body>, Error> {
    let entrants = state.store.get_entrants(id).await?;

    let body = serde_json::to_vec(&entrants)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))
        .unwrap())
}

async fn get(
    _req: Request,
    state: State,
    tournament_id: TournamentId,
    id: EntrantId,
) -> Result<Response<Body>, Error> {
    let entrant = state.store.get_entrant(tournament_id, id).await?;

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

async fn create(
    req: Request,
    state: State,
    tournament_id: TournamentId,
) -> Result<Response<Body>, Error> {
    let tournament = state.store.get_tournament(tournament_id).await?;

    let body: Entrant = req.json().await?;

    let mut resp = Response::new(Body::empty());
    match tournament {
        Some(tournament) => {
            if tournament.kind == body.kind() {
                let id = state.store.insert_entrant(tournament_id, body).await?;

                *resp.status_mut() = StatusCode::OK;
                *resp.body_mut() = Body::from(id.to_string());
            } else {
                *resp.status_mut() = StatusCode::BAD_REQUEST;
                *resp.body_mut() = Body::from("invalid entrant kind for this tournament");
            }
        }
        None => {
            *resp.status_mut() = StatusCode::NOT_FOUND;
            *resp.body_mut() = Body::from("invalid tournament id");
        }
    }

    Ok(resp)
}
