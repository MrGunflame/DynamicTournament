use crate::http::{Request, RequestUri};
use crate::{Error, State, StatusCodeError};

use dynamic_tournament_api::tournament::{BracketType, TournamentId, TournamentOverview};
use dynamic_tournament_api::v3::id::SystemId;
use hyper::{Body, Method, Response, StatusCode};

pub async fn route(
    req: Request,
    mut uri: RequestUri<'_>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take() {
        None => match *req.method() {
            Method::GET => list(req, state).await,
            Method::POST => create(req, state).await,
            Method::OPTIONS => Ok(Response::builder()
                .status(204)
                .body(Body::from("No Content"))
                .unwrap()),
            _ => Err(StatusCodeError::method_not_allowed().into()),
        },
        Some(id) => {
            let id: u64 = id.parse()?;

            match uri.take_str() {
                None => match *req.method() {
                    Method::GET => get(req, id, state).await,
                    Method::OPTIONS => Ok(Response::builder()
                        .status(204)
                        .body(Body::from("No Content"))
                        .unwrap()),
                    _ => Err(StatusCodeError::method_not_allowed().into()),
                },
                _ => Err(StatusCodeError::not_found().into()),
            }
        }
    }
}

async fn list(_req: Request, state: State) -> Result<Response<Body>, Error> {
    let tournaments = state.store.list_tournaments().await?;
    let mut entrants = Vec::with_capacity(tournaments.len());
    let mut brackets = Vec::with_capacity(tournaments.len());

    for tournament in tournaments.iter() {
        let e = state.store.get_entrants(tournament.id).await?;
        entrants.push(e.len() as u64);

        let b = state.store.list_brackets(tournament.id).await?;
        brackets.push(if b.is_empty() {
            // Placeholder
            BracketType::SingleElimination
        } else {
            match b[0].system {
                SystemId(1) => BracketType::SingleElimination,
                SystemId(2) => BracketType::DoubleElimination,
                _ => unreachable!(),
            }
        });
    }

    let body: Vec<TournamentOverview> = tournaments
        .into_iter()
        .zip(entrants.into_iter())
        .zip(brackets.into_iter())
        .map(|((t, e), b)| TournamentOverview {
            id: TournamentId(t.id.0),
            name: t.name,
            date: t.date,
            bracket_type: b,
            entrants: e,
        })
        .collect();

    let body = serde_json::to_string(&body)?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();

    Ok(resp)
}

async fn create(req: Request, state: State) -> Result<Response<Body>, Error> {
    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = StatusCode::SERVICE_UNAVAILABLE;

    Ok(resp)
}

async fn get(_req: Request, id: u64, state: State) -> Result<Response<Body>, Error> {
    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = StatusCode::SERVICE_UNAVAILABLE;

    Ok(resp)
}
