use dynamic_tournament_api::v3::id::{RoleId, TournamentId};
use hyper::{header::CONTENT_TYPE, Body, Method, Response, StatusCode};

use crate::method;
use crate::{
    http::{Request, RequestUri},
    Error, State, StatusCodeError,
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
    let roles = state.store.get_roles(id).await?;

    let body = serde_json::to_vec(&roles)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap())
}

async fn get(
    _req: Request,
    state: State,
    tournament_id: TournamentId,
    id: RoleId,
) -> Result<Response<Body>, Error> {
    let role = state.store.get_role(id, tournament_id).await?;
    if role.is_none() {
        return Err(StatusCodeError::not_found().into());
    }

    let body = serde_json::to_vec(&role.unwrap())?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap())
}

async fn create(
    req: Request,
    state: State,
    tournament_id: TournamentId,
) -> Result<Response<Body>, Error> {
    let tournament = state.store.get_tournament(tournament_id).await?;
    if tournament.is_none() {
        return Err(StatusCodeError::not_found().into());
    }

    if !state.is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    let body = req.json().await?;

    state.store.insert_role(body, tournament_id).await?;

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::empty())
        .unwrap())
}
