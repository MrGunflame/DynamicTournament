pub mod auth;
mod tournament;

use dynamic_tournament_macros::path;

use crate::http::{Request, RequestUri, Result};

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    path!(uri, {
        "tournament" => tournament::route(req, uri).await,
        "auth" => auth::route(req, uri).await,
    })
}
