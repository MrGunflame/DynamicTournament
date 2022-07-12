use dynamic_tournament_api::v3::id::{RoleId, TournamentId};
use dynamic_tournament_api::v3::tournaments::roles::Role;
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

            method!(req, {
                Method::GET => get(req, tournament_id, id).await,
                Method::DELETE => delete(req, tournament_id, id).await,
            })
        }
    }
}

async fn list(req: Request, id: TournamentId) -> Result {
    let roles = req.state().store.roles(id).list().await?;

    Ok(Response::ok().json(&roles))
}

async fn get(req: Request, tournament_id: TournamentId, id: RoleId) -> Result {
    let role = req.state().store.roles(tournament_id).get(id).await?;

    match role {
        Some(role) => Ok(Response::ok().json(&role)),
        None => Err(StatusCodeError::not_found().into()),
    }
}

async fn create(mut req: Request, tournament_id: TournamentId) -> Result {
    req.require_authentication()?;

    let mut role: Role = req.json().await?;

    role.id = req.state().store.roles(tournament_id).insert(&role).await?;

    Ok(Response::created().json(&role))
}

async fn delete(req: Request, tournament_id: TournamentId, id: RoleId) -> Result {
    req.require_authentication()?;

    req.state().store.roles(tournament_id).delete(id).await?;

    Ok(Response::ok())
}
