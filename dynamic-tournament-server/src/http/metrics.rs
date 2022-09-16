use crate::http::{Context, Response, Result};

use dynamic_tournament_macros::method;

pub async fn route(ctx: Context) -> Result {
    method!(ctx, {
        GET => get(ctx).await,
    })
}

async fn get(ctx: Context) -> Result {
    let body = ctx.state.metrics.serialize();

    Ok(Response::ok().body(body))
}
