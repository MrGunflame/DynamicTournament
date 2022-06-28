use crate::{Error, StatusCodeError};

use hyper::{Body, Response};

pub async fn route() -> Result<Response<Body>, Error> {
    Err(StatusCodeError::gone()
        .message("v1 is depreciated. Use v2 instead")
        .into())
}
