use crate::http::{Request, Response, Result};
use crate::method;

use hyper::Method;

pub async fn route(req: Request) -> Result {
    method!(req, {
        Method::GET => get(req).await,
    })
}

async fn get(req: Request) -> Result {
    let body = req.state.metrics.serialize();

    Ok(Response::ok().body(body))
}
