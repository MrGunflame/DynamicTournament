mod matches;

use dynamic_tournament_api::v3::{
    id::{BracketId, SystemId, TournamentId},
    tournaments::brackets::Bracket,
};
use dynamic_tournament_generator::{options::TournamentOptions, EntrantScore, SingleElimination};
use hyper::{Body, Method, Response, StatusCode};

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

async fn list(req: Request, id: TournamentId) -> Result<Response<Body>, Error> {
    let brackets = req.state().store.list_brackets(id).await?;

    let resp = Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&brackets)?))
        .unwrap();

    Ok(resp)
}

async fn get(
    req: Request,
    tournament_id: TournamentId,
    id: BracketId,
) -> Result<Response<Body>, Error> {
    let bracket = req.state().store.get_bracket(tournament_id, id).await?;

    let bracket = match bracket {
        Some(bracket) => bracket,
        None => return Err(StatusCodeError::not_found().into()),
    };

    Ok(Response::new(Body::from(serde_json::to_vec(&bracket)?)))
}

async fn create(mut req: Request, tournament_id: TournamentId) -> Result<Response<Body>, Error> {
    if !req.state().is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    let mut bracket: Bracket = req.json().await?;

    // Make sure all entrants in the bracket actually exist.
    let entrants = req.state().store.get_entrants(tournament_id).await?;

    // Keep track of consumed ids to deny duplicates.
    let mut consumed = Vec::with_capacity(bracket.entrants.len());

    let mut resp = Response::new(Body::empty());

    for id in bracket.entrants.iter() {
        if consumed.contains(id) {
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            *resp.body_mut() = Body::from(format!("found entrant {} multiple times", id));

            return Ok(resp);
        }

        if !entrants.iter().any(|e| e.id == *id) {
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            *resp.body_mut() = Body::from(format!(
                "invalid entrant {}, does not exist for tournament",
                id
            ));

            return Ok(resp);
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

    Ok(Response::new(Body::from(id.to_string())))
}
