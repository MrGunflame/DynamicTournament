#[cfg(feature = "server")]
mod server;

use proc_macro::TokenStream;

/// Match the HTTP request method of a request.
///
/// The following request methods are allowed:
/// - `GET`
/// - `POST`
/// - `PATCH`
/// - `PUT`
/// - `DELETE`
///
/// The `HEAD` and `OPTIONS` implementations are automatically generated. All other missing method
/// will be responded with a `405 Method Not Allowed` status.
///
/// # Examples
/// ```ignore
/// # use dynamic_tournament_macros::method;
/// #
/// async fn route(req: Request) -> Result<Response> {
///     method!(req, {
///         GET => get(req).await,
///     })
/// }
///
/// async fn get(req: Request) -> Result<Response> {
///     Ok(Response::ok())
/// }
/// ```
#[cfg(feature = "server")]
#[proc_macro]
pub fn method(input: TokenStream) -> TokenStream {
    server::method(input)
}
