use crate::http::{Request, RequestUri};
use crate::{Error, State, StatusCodeError};

use dynamic_tournament_api::tournament::{
    BracketType, Entrants, Player, Role, Team, Tournament, TournamentId, TournamentOverview,
};
use dynamic_tournament_api::v3::id::{RoleId, SystemId};
use dynamic_tournament_api::v3::tournaments::brackets::Bracket as Bracket2;
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use dynamic_tournament_api::v3::tournaments::entrants::{Player as Player2, Team as Team2};
use dynamic_tournament_api::v3::tournaments::{EntrantKind, Tournament as Tournament2};
use dynamic_tournament_generator::options::TournamentOptionValues;
use dynamic_tournament_generator::{EntrantScore, SingleElimination};
use hyper::header::HeaderValue;
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
    if !state.is_authenticated(&req) {
        return Err(StatusCodeError::unauthorized().into());
    }

    let body: Tournament = req.json().await?;

    let kind = match body.entrants {
        Entrants::Players(_) => EntrantKind::Player,
        Entrants::Teams(_) => EntrantKind::Team,
    };

    let tournament = Tournament2 {
        id: 0.into(),
        name: body.name,
        description: body.description,
        date: body.date,
        kind,
    };

    let entrants: Vec<Entrant> = match body.entrants {
        Entrants::Players(players) => players
            .into_iter()
            .map(|p| Entrant {
                id: 0.into(),
                inner: EntrantVariant::Player(Player2 {
                    rating: p.rating,
                    name: p.name,
                    role: RoleId(0),
                }),
            })
            .collect(),
        Entrants::Teams(teams) => teams
            .into_iter()
            .map(|t| Entrant {
                id: 0.into(),
                inner: EntrantVariant::Team(Team2 {
                    name: t.name,
                    players: t
                        .players
                        .into_iter()
                        .map(|p| Player2 {
                            rating: p.rating,
                            name: p.name,
                            role: RoleId(0),
                        })
                        .collect(),
                }),
            })
            .collect(),
    };

    let id = state.store.insert_tournament(&tournament).await?;

    let mut entrant_ids = Vec::new();
    for entrant in entrants {
        let id = state.store.insert_entrant(id, entrant).await?;
        entrant_ids.push(id);
    }

    let bracket = Bracket2 {
        id: 0.into(),
        name: tournament.name.clone(),
        system: match body.bracket_type {
            BracketType::SingleElimination => SystemId(1),
            BracketType::DoubleElimination => SystemId(2),
        },
        options: match body.bracket_type {
            BracketType::SingleElimination => TournamentOptionValues::default()
                .merge(SingleElimination::<u8, EntrantScore<u8>>::options())
                .unwrap(),
            BracketType::DoubleElimination => TournamentOptionValues::default(),
        },
        entrants: entrant_ids,
    };

    state.store.insert_bracket(id, &bracket).await?;

    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = StatusCode::CREATED;
    *resp.body_mut() = Body::from(id.to_string());

    Ok(resp)
}

async fn get(_req: Request, id: u64, state: State) -> Result<Response<Body>, Error> {
    let tournament = match state.store.get_tournament(id.into()).await? {
        Some(t) => t,
        None => return Err(StatusCodeError::not_found().into()),
    };

    let entrants = state.store.get_entrants(id.into()).await?;

    let entrants = match entrants.get(0).cloned() {
        Some(e) => match e.inner {
            EntrantVariant::Player(_) => Entrants::Players(
                entrants
                    .into_iter()
                    .map(|p| match p.inner {
                        EntrantVariant::Player(p) => Player {
                            name: p.name,
                            role: Role::Unknown,
                            rating: p.rating,
                        },
                        EntrantVariant::Team(_) => unreachable!(),
                    })
                    .collect(),
            ),
            EntrantVariant::Team(_) => Entrants::Teams(
                entrants
                    .into_iter()
                    .map(|t| match t.inner {
                        EntrantVariant::Player(_) => unreachable!(),
                        EntrantVariant::Team(t) => Team {
                            name: t.name,
                            players: t
                                .players
                                .into_iter()
                                .map(|p| Player {
                                    name: p.name,
                                    rating: p.rating,
                                    role: Role::Unknown,
                                })
                                .collect(),
                        },
                    })
                    .collect(),
            ),
        },
        None => Entrants::Teams(Vec::new()),
    };

    let body = serde_json::to_vec(&Tournament {
        id: TournamentId(id),
        name: tournament.name,
        description: tournament.description,
        date: tournament.date,
        bracket_type: BracketType::SingleElimination,
        entrants,
    })?;

    let mut resp = Response::new(Body::empty());
    resp.headers_mut()
        .append("Content-Type", HeaderValue::from_static("application/json"));

    *resp.body_mut() = Body::from(body);

    Ok(resp)
}
