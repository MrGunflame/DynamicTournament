use dynamic_tournament_api::auth::Flags;
use dynamic_tournament_api::v3::id::TournamentId;
use dynamic_tournament_macros::{method, path};

use crate::http::{Context, Response, Result};

pub async fn route(mut ctx: Context, id: TournamentId) -> Result {
    path!(ctx, {
        @ => method!(ctx, {
            GET => list(ctx, id).await,
        })
    })
}

async fn list(ctx: Context, id: TournamentId) -> Result {
    ctx.require_authentication(Flags::ADMIN)?;

    let events = ctx.state.store.event_log(id).list().await?;
    Ok(Response::ok().json(&events))
}
