use dynamic_tournament_api::{auth::Flags, v3::id::TournamentId};

use crate::http::{method, path, Context, Response, Result};

pub async fn route(mut ctx: Context, id: TournamentId) -> Result {
    path!(ctx, {
        @ => method!(ctx, {
            GET => list(ctx, id).await,
        })
    })
}

async fn list(ctx: Context, id: TournamentId) -> Result {
    ctx.require_authentication(Flags::ADMIN)?;

    let entries = ctx.state.store.log(id).list().await?;

    Ok(Response::ok().json(&entries))
}
