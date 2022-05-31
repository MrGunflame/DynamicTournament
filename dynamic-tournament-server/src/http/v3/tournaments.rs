use dynamic_tournament_api::v3::{id::TournamentId, tournaments::TournamentOverview};
use hyper::{Body, Method, Response, StatusCode};

use crate::{
    http::{Request, RequestUri},
    Error, State, StatusCodeError,
};

pub async fn route(
    req: Request,
    mut uri: RequestUri<'_>,
    state: State,
) -> Result<Response<Body>, Error> {
    match uri.take() {
        None => match *req.method() {
            Method::GET => list(req, state).await,
            _ => Err(StatusCodeError::method_not_allowed().into()),
        },
        Some(part) => {
            let id = part.parse()?;

            match *req.method() {
                Method::GET => get(req, state, id).await,
                _ => Err(StatusCodeError::method_not_allowed().into()),
            }
        }
    }
}

async fn list(_req: Request, _state: State) -> Result<Response<Body>, Error> {
    let tournaments: Vec<TournamentOverview> = vec![];

    let body = serde_json::to_vec(&tournaments)?;

    let resp = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap();

    Ok(resp)
}

async fn get(_req: Request, _state: State, _id: TournamentId) -> Result<Response<Body>, Error> {
    unimplemented!()
}
