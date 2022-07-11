use dynamic_tournament_core::options::TournamentOptions;
use hyper::Method;

use crate::http::{Request, RequestUri, Response, Result};
use crate::{method, StatusCodeError};

use dynamic_tournament_api::v3::id::SystemId;
use dynamic_tournament_api::v3::systems::{System, SystemOverview};
use dynamic_tournament_core::{EntrantScore, SingleElimination};

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    match uri.take() {
        None => method!(req, {
            Method::GET => list(req).await,
        }),
        Some(part) => {
            let id = part.parse()?;

            method!(req, {
                Method::GET => get(req, id).await,
            })
        }
    }
}

async fn list(_req: Request) -> Result {
    // Hardcoded for now.
    let systems = [
        SystemOverview {
            id: SystemId(1),
            name: "Single Elimination".into(),
        },
        SystemOverview {
            id: SystemId(2),
            name: "Double Elimination".into(),
        },
    ];

    Ok(Response::ok().json(&systems))
}

async fn get(_req: Request, id: SystemId) -> Result {
    let system = match id.as_ref() {
        1 => Some(System {
            id: SystemId(1),
            name: "Single Elimination".into(),
            options: SingleElimination::<u8, EntrantScore<u8>>::options(),
        }),
        2 => Some(System {
            id: SystemId(2),
            name: "Double Elimination".into(),
            options: TournamentOptions::default(),
        }),
        _ => None,
    };

    match system {
        Some(system) => Ok(Response::ok().json(&system)),
        None => Err(StatusCodeError::not_found().into()),
    }
}
