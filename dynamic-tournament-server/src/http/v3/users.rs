use dynamic_tournament_api::v3::users::User;
use dynamic_tournament_macros::method;
use snowflaked::sync::Generator;

use crate::auth::password_hash;
use crate::http::{Request, RequestUri, Response, Result, StatusCodeError};

pub static USER_ID_GENERATOR: Generator = Generator::new_unchecked(0);

pub async fn route(req: Request, mut uri: RequestUri<'_>) -> Result {
    match uri.take_str() {
        None => {
            method!(req, {
                POST => create(req).await,
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
    user.password = password_hash(&user.password, user.id.0.to_le_bytes());
}
