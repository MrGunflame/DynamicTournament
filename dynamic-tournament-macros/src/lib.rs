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

/// Match the next segment of a request uri.
///
/// # Examples
/// ```ignore
/// # use dynamic_tournament_macros::path;
/// #
/// async fn route(ctx: Context) -> Result {
///     path!(ctx, {
///         // The `@` token matches an empty segment.
///         @ => index().await,
///         // A string literal matches if the segment is the exact literal.
///         "users" => users().await,
///         // A variable name causes the segment to be parsed into the variable.
///         // If the parsing fails an 404 Not Found error is returned.
///         name => greet(name).await,
///         // Optionally give the variable a type hint.
///         name: String => greet_generic(name).await,
///     })
/// }
///
/// async fn index() -> Result {
///     Ok(Response::ok().body("Matched route: /"))
/// }
///
/// async fn users() -> Result {
///     Ok(Response::ok().body("Matched route: /users"))
/// }
///
/// async fn greet(name: String) -> Result {
///     Ok(Response::ok().body(format!("Hello {}", name)))
/// }
///
/// async fn greet_generic<T: AsRef<str>>(name: T) -> Result {
///     Ok(Response::ok().body(format!("Generic hello {}", name.as_ref())))
/// }
/// ```
#[cfg(feature = "server")]
#[proc_macro]
pub fn path(input: TokenStream) -> TokenStream {
    server::path(input)
}
