use std::collections::HashMap;

use crate::http::{Request, RequestUri, Response, Result};
use crate::StatusCodeError;

use dynamic_tournament_api::tournament::{
    BracketType, Entrants, Player, Role, Team, Tournament, TournamentId, TournamentOverview,
};
use dynamic_tournament_api::v3::id::SystemId;
use dynamic_tournament_api::v3::tournaments::brackets::Bracket as Bracket2;
use dynamic_tournament_api::v3::tournaments::entrants::{Entrant, EntrantVariant};
use dynamic_tournament_api::v3::tournaments::entrants::{Player as Player2, Team as Team2};
use dynamic_tournament_api::v3::tournaments::roles::Role as Role2;
use dynamic_tournament_api::v3::tournaments::{EntrantKind, Tournament as Tournament2};
use dynamic_tournament_core::options::TournamentOptionValues;
use dynamic_tournament_core::{EntrantScore, SingleElimination};
use dynamic_tournament_macros::method;

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    match uri.take() {
        None => method!(req, {
            GET => list(req).await,
            POST => create(req).await,
        }),
        Some(id) => {
            let id: u64 = id.parse()?;

            method!(req, {
                GET => get(req, id).await,
            })
        }
    }
}

async fn list(req: Request) -> Result {
    let tournaments = req.state().store.list_tournaments().await?;
    let mut entrants = Vec::with_capacity(tournaments.len());
    let mut brackets = Vec::with_capacity(tournaments.len());

    for tournament in tournaments.iter() {
        let e = req.state().store.get_entrants(tournament.id).await?;
        entrants.push(e.len() as u64);

        let b = req.state().store.list_brackets(tournament.id).await?;
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

    Ok(Response::ok().json(&body))
}

async fn create(mut req: Request) -> Result {
    req.require_authentication()?;

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

    let id = req.state().store.insert_tournament(&tournament).await?;

    let mut roles = HashMap::new();
    for role in [
        Role::Roamer,
        Role::Teamfighter,
        Role::Duelist,
        Role::Support,
    ] {
        let id = req
            .state()
            .store
            .roles(id)
            .insert(&Role2 {
                id: 0.into(),
                name: role.to_string(),
            })
            .await?;
        roles.insert(role, id);
    }

    let entrants: Vec<Entrant> = match body.entrants {
        Entrants::Players(players) => players
            .into_iter()
            .map(|p| Entrant {
                id: 0.into(),
                inner: EntrantVariant::Player(Player2 {
                    rating: p.rating,
                    name: p.name,
                    role: *roles.get(&p.role).unwrap(),
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
                            role: *roles.get(&p.role).unwrap(),
                        })
                        .collect(),
                }),
            })
            .collect(),
    };

    let mut entrant_ids = Vec::new();
    for entrant in entrants {
        let id = req.state().store.insert_entrant(id, entrant).await?;
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

    req.state().store.insert_bracket(id, &bracket).await?;

    Ok(Response::created().body(id.to_string()))
}

async fn get(req: Request, id: u64) -> Result {
    let tournament = match req.state().store.get_tournament(id.into()).await? {
        Some(t) => t,
        None => return Err(StatusCodeError::not_found().into()),
    };

    let entrants = req.state().store.get_entrants(id.into()).await?;

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

    let body = Tournament {
        id: TournamentId(id),
        name: tournament.name,
        description: tournament.description,
        date: tournament.date,
        bracket_type: BracketType::SingleElimination,
        entrants,
    };

    Ok(Response::ok().json(&body))
}
