use dynamic_tournament_api::v3::id::{RoleId, TournamentId};
use dynamic_tournament_api::v3::tournaments::roles::Role;
use dynamic_tournament_api::Payload;
use dynamic_tournament_macros::{method, path};

use crate::{
    http::{Request, RequestUri, Response, Result},
    StatusCodeError,
};

pub async fn route(req: Request, mut uri: RequestUri<'_>, tournament_id: TournamentId) -> Result {
    path!(uri, {
        @ => method!(req, {
            GET => list(req, tournament_id).await,
            POST => create(req, tournament_id).await,
        }),
        id => method!(req, {
            GET => get(req, tournament_id, id).await,
            DELETE => delete(req, tournament_id, id).await,
        })
    })
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

    let mut roles: Payload<Role> = req.json().await?;

    for role in roles.iter_mut() {
        role.id = req.state().store.roles(tournament_id).insert(role).await?;
    }

    Ok(Response::created().json(&roles))
}

async fn delete(req: Request, tournament_id: TournamentId, id: RoleId) -> Result {
    req.require_authentication()?;

    req.state().store.roles(tournament_id).delete(id).await?;

    Ok(Response::ok())
}
