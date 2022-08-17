use dynamic_tournament_api::v3::users::User;
use hyper::Method;
use sha2::{Digest, Sha512};
use snowflaked::sync::Generator;

use crate::http::{Request, RequestUri, Response, Result, StatusCodeError};
use crate::method;

static USER_ID_GENERATOR: Generator = Generator::new_unchecked(0);

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    match uri.take_str() {
        None => {
            method!(req, {
                Method::POST => create(req).await,
            })
        }
        _ => Err(StatusCodeError::not_found().into()),
    }
}

async fn create(mut req: Request) -> Result {
    req.require_authentication()?;

    let mut user: User = req.json().await?;

    user.id.0 = USER_ID_GENERATOR.generate();
    apply_hash(&mut user);

    req.state().store.users().insert(&user).await?;

    Ok(Response::ok())
}

/// Apply the password hash to the user.
/// See v2/auth.rs for hashing details.
fn apply_hash(user: &mut User) {
    let mut hasher = Sha512::new();
    hasher.update(user.password.as_bytes());
    hasher.update(user.id.0.to_le_bytes());

    let res = hasher.finalize();
    let hash = hex::encode(res);

    user.password = hash;
}
