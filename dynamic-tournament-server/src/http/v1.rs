use crate::http::{Request, RequestUri};
use crate::{Error, State};

use hyper::{Body, Response, StatusCode};

pub async fn route() -> Result<Response<Body>, Error> {
    let mut resp = Response::new(Body::empty());
    *resp.status_mut() = StatusCode::GONE;
    *resp.body_mut() = Body::from("v1 is depreciated. Use v2 instead");

    Ok(resp)
}
