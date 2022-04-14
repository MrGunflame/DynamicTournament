pub mod auth;
pub mod client;

pub use auth::AuthProvider;
pub use client::ClientProvider;
use yew::{Component, Context};

pub trait Provider<T, C>: Component
where
    C: Component,
{
    /// Takes a new context `T` from the provider.
    fn take(ctx: &Context<C>) -> T;
}
