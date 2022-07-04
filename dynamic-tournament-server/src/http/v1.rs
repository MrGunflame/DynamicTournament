use crate::http::Result;
use crate::StatusCodeError;

pub async fn route() -> Result {
    Err(StatusCodeError::gone()
        .message("v1 is depreciated. Use v2 instead")
        .into())
}
