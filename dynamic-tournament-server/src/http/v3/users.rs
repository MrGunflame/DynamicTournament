use dynamic_tournament_api::v3::users::User;
use dynamic_tournament_macros::{method, path};
use snowflaked::sync::Generator;

use crate::auth::password_hash;
use crate::http::{Context, Response, Result};

pub static USER_ID_GENERATOR: Generator = Generator::new_unchecked(0);

pub async fn route(mut ctx: Context) -> Result {
    path!(ctx, {
        @ => {
            method!(ctx, {
                POST => create(ctx).await,
            })
        }
    })
}

async fn create(mut ctx: Context) -> Result {
    ctx.require_authentication()?;

    let mut user: User = ctx.req.json().await?;

    user.id.0 = USER_ID_GENERATOR.generate();
    apply_hash(&mut user);

    ctx.state.store.users().insert(&user).await?;

    Ok(Response::ok())
}

/// Apply the password hash to the user.
/// See v2/auth.rs for hashing details.
fn apply_hash(user: &mut User) {
    user.password = password_hash(&user.password, user.id.0.to_le_bytes());
}
