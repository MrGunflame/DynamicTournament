mod systems;
mod tournaments;
pub mod users;

use dynamic_tournament_macros::path;

use crate::http::{Context, Result};

use super::v2;

pub async fn route(mut ctx: Context) -> Result {
    path!(ctx, {
        // /v3/auth uses the same endpoint as /v2/auth.
        "auth" => v2::auth::route(ctx).await,
        "tournaments" => tournaments::route(ctx).await,
        "systems" => systems::route(ctx).await,
        "users" => users::route(ctx).await,
    })
}
