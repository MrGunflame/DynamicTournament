use crate::http::{Request, Response, Result};

use dynamic_tournament_macros::method;

pub async fn route(req: Request) -> Result {
    method!(req, {
        GET => get(req).await,
    })
}

async fn get(req: Request) -> Result {
    let body = req.state.metrics.serialize();

    Ok(Response::ok().body(body))
}
