pub mod auth;
mod tournament;

use dynamic_tournament_macros::path;

use crate::http::{Context, Result};

pub async fn route(mut ctx: Context) -> Result {
    path!(ctx, {
        "tournament" => tournament::route(ctx).await,
        "auth" => auth::route(ctx).await,
    })
}
