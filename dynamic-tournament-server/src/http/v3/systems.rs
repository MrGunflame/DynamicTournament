use dynamic_tournament_core::options::TournamentOptions;

use crate::http::{Context, Response, Result};
use crate::StatusCodeError;

use dynamic_tournament_api::v3::id::SystemId;
use dynamic_tournament_api::v3::systems::{System, SystemOverview};
use dynamic_tournament_core::{EntrantScore, SingleElimination};
use dynamic_tournament_macros::{method, path};

pub async fn route(mut ctx: Context) -> Result {
    path!(ctx, {
        @ => method!(ctx, {
            GET => list(ctx).await,
        }),
        id => method!(ctx, {
            GET => get(ctx, id).await,
        }),
    })
}

async fn list(_ctx: Context) -> Result {
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
        SystemOverview {
            id: SystemId(3),
            name: "Round Robin".into(),
        },
    ];

    Ok(Response::ok().json(&systems))
}

async fn get(_ctx: Context, id: SystemId) -> Result {
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
        3 => Some(System {
            id: SystemId(3),
            name: "Round Robin".into(),
            options: TournamentOptions::default(),
        }),
        _ => None,
    };

    match system {
        Some(system) => Ok(Response::ok().json(&system)),
        None => Err(StatusCodeError::not_found().into()),
    }
}
