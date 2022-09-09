mod systems;
mod tournaments;
pub mod users;

use dynamic_tournament_macros::path;

use crate::http::{Request, RequestUri, Result};

use super::v2;

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    path!(uri, {
        // /v3/auth uses the same endpoint as /v2/auth.
        "auth" => v2::auth::route(req, uri).await,
        "tournaments" => tournaments::route(req, uri).await,
        "systems" => systems::route(req, uri).await,
        "users" => users::route(req, uri).await,
    })
}
