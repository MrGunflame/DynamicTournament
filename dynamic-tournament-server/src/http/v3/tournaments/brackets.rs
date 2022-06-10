mod matches;

use dynamic_tournament_api::v3::{
    id::{BracketId, SystemId, TournamentId},
    tournaments::brackets::Bracket,
};
use dynamic_tournament_generator::{options::TournamentOptions, EntrantScore, SingleElimination};
use hyper::{Body, Method, Response, StatusCode};

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
        None => match *req.method() {
            Method::GET => list(req, state, tournament_id).await,
            Method::POST => create(req, state, tournament_id).await,
            _ => Err(StatusCodeError::method_not_allowed().into()),
        },
        Some(part) => {
            let id = part.parse()?;

            match uri.take_str() {
                None => match *req.method() {
                    Method::GET => get(req, state, tournament_id, id).await,
                    _ => Err(StatusCodeError::method_not_allowed().into()),
                },
                Some("matches") => matches::route(req, uri, state, tournament_id).await,
                Some(_) => Err(StatusCodeError::not_found().into()),
            }
        }
    }
}

async fn list(_req: Request, state: State, id: TournamentId) -> Result<Response<Body>, Error> {
    let brackets = state.store.list_brackets(id).await?;

    let resp = Response::builder()
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&brackets)?))
        .unwrap();

    Ok(resp)
}

async fn get(
    _req: Request,
    state: State,
    tournament_id: TournamentId,
    id: BracketId,
) -> Result<Response<Body>, Error> {
    let bracket = state.store.get_bracket(tournament_id, id).await?;

    let bracket = match bracket {
        Some(bracket) => bracket,
        None => return Err(StatusCodeError::not_found().into()),
    };

    Ok(Response::new(Body::from(serde_json::to_vec(&bracket)?)))
}

async fn create(
    req: Request,
    state: State,
    tournament_id: TournamentId,
) -> Result<Response<Body>, Error> {
    if !state.is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    let mut bracket: Bracket = req.json().await?;

    // Make sure all entrants in the bracket actually exist.
    let entrants = state.store.get_entrants(tournament_id).await?;

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

    let id = state.store.insert_bracket(tournament_id, &bracket).await?;

    Ok(Response::new(Body::from(id.to_string())))
}
