mod brackets;
mod entrants;
mod roles;

use dynamic_tournament_api::v3::id::TournamentId;
use hyper::{Body, Method, Response, StatusCode};

use crate::method;
use crate::{
    http::{Request, RequestUri},
    Error, StatusCodeError,
};

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result<Response<Body>, Error> {
    match uri.take() {
        None => method!(req, {
            Method::GET => list(req).await,
            Method::POST => create(req).await,
        }),
        Some(part) => {
            let id = part.parse()?;

            // Check if the tournament exists before continuing.
            if req.state().store.tournaments().get(id).await?.is_none() {
                return Err(StatusCodeError::not_found()
                    .message("Invalid tournament id")
                    .into());
            }

            match uri.take_str() {
                Some("entrants") => entrants::route(req, uri, id).await,
                Some("brackets") => brackets::route(req, uri, id).await,
                Some("roles") => roles::route(req, uri, id).await,
                None => method!(req, {
                    Method::GET => get(req, id).await,
                    Method::PATCH => patch(req, id).await,
                    Method::DELETE => delete(req, id).await,
                }),
                Some(_) => Err(StatusCodeError::not_found().into()),
            }
        }
    }
}

async fn list(req: Request) -> Result<Response<Body>, Error> {
    let tournaments = req.state().store.tournaments().list().await?;

    let body = serde_json::to_vec(&tournaments)?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();

    Ok(resp)
}

async fn get(req: Request, id: TournamentId) -> Result<Response<Body>, Error> {
    let tournament = req.state().store.tournaments().get(id).await?;

    let tournament = tournament.ok_or_else(StatusCodeError::not_found)?;

    Ok(Response::new(Body::from(serde_json::to_vec(&tournament)?)))
}

async fn create(mut req: Request) -> Result<Response<Body>, Error> {
    if !req.state().is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    let tournament = req.json().await?;

    let id = req.state().store.tournaments().insert(&tournament).await?;

    Ok(Response::new(Body::from(id.to_string())))
}

async fn patch(mut req: Request, id: TournamentId) -> Result<Response<Body>, Error> {
    if !req.state().is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    // Check if the tournament exists.
    let mut tournament = match req.state().store.tournaments().get(id).await? {
        Some(tournament) => tournament,
        None => return Err(StatusCodeError::not_found().into()),
    };

    let partial = req.json().await?;
    req.state().store.tournaments().update(id, &partial).await?;

    // Merge the patch.
    tournament.update(partial);

    Ok(Response::new(Body::from(serde_json::to_vec(&tournament)?)))
}

async fn delete(req: Request, id: TournamentId) -> Result<Response<Body>, Error> {
    if !req.state().is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    req.state().store.tournaments().delete(id).await?;

    Ok(Response::new(Body::empty()))
}
