use dynamic_tournament_api::v3::id::{RoleId, TournamentId};
use hyper::{header::CONTENT_TYPE, Body, Method, Response, StatusCode};

use crate::method;
use crate::{
    http::{Request, RequestUri},
    Error, StatusCodeError,
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
                Method::DELETE => delete(req, tournament_id, id).await,
            })
        }
    }
}

async fn list(req: Request, id: TournamentId) -> Result<Response<Body>, Error> {
    let roles = req.state().store.roles(id).list().await?;

    let body = serde_json::to_vec(&roles)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap())
}

async fn get(
    req: Request,
    tournament_id: TournamentId,
    id: RoleId,
) -> Result<Response<Body>, Error> {
    let role = req.state().store.roles(tournament_id).get(id).await?;
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

async fn create(mut req: Request, tournament_id: TournamentId) -> Result<Response<Body>, Error> {
    if !req.state().is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    let body = req.json().await?;

    req.state().store.insert_role(body, tournament_id).await?;

    Ok(Response::builder()
        .status(StatusCode::CREATED)
        .body(Body::empty())
        .unwrap())
}

async fn delete(
    req: Request,
    tournament_id: TournamentId,
    id: RoleId,
) -> Result<Response<Body>, Error> {
    if !req.state().is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    req.state().store.roles(tournament_id).delete(id).await?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap())
}
