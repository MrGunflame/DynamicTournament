use dynamic_tournament_api::v3::{id::TournamentId, tournaments::brackets::Bracket};
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
            //let id = part.parse()?;

            match *req.method() {
                //Method::GET => get(req, state, tournament_id, id).await,
                _ => Err(StatusCodeError::method_not_allowed().into()),
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

async fn create(
    req: Request,
    state: State,
    tournament_id: TournamentId,
) -> Result<Response<Body>, Error> {
    if !state.is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    let bracket: Bracket = req.json().await?;

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

        if entrants.iter().find(|&e| e.id == *id).is_none() {
            *resp.status_mut() = StatusCode::BAD_REQUEST;
            *resp.body_mut() = Body::from(format!(
                "invalid entrant {}, does not exist for tournament",
                id
            ));

            return Ok(resp);
        }

        consumed.push(*id);
    }

    let id = state.store.insert_bracket(tournament_id, &bracket).await?;

    Ok(Response::new(Body::from(id.to_string())))
}
