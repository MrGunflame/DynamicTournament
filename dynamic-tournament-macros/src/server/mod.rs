mod path;

use std::collections::HashMap;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, Expr, Ident, Result, Token};

pub use path::path;

pub fn method(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as MethodRoot);
    TokenStream::from(input.expand())
}

struct MethodRoot {
    ctx: Expr,
    branches: HashMap<Method, Expr>,
}

impl MethodRoot {
    fn expand_head(&self) -> TokenStream2 {
        match self.branches.get(&Method::Get) {
            Some(branch) => {
                quote! {
                    method if method == hyper::Method::HEAD => {
                        let mut res = { #branch };
                        res.map(|resp| {
                            resp.body(hyper::Body::empty())
                        })
                    }
                }
            }
            None => {
                // Will fall through to default 405 Method not allowed status.
                quote! {}
            }
        }
    }

    fn expand_options(&self) -> TokenStream2 {
        let methods: String = self
            .branches
            .keys()
            .map(|method| method.as_str())
            .collect::<Vec<&str>>()
            .join(",");

        quote! {
            method if method == hyper::Method::OPTIONS => {
                use crate::http::Response;
                use hyper::header::{HeaderValue, ALLOW, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_HEADERS};

                let allow = HeaderValue::from_static(#methods);
                let headers  = HeaderValue::from_static("X-WP-Nonce");

                Ok(Response::no_content()
                    .header(ALLOW, allow.clone())
                    .header(ACCESS_CONTROL_ALLOW_METHODS, allow)
                    .header(ACCESS_CONTROL_ALLOW_HEADERS, headers))
            }
        }
    }

    fn expand(self) -> TokenStream2 {
        let head = self.expand_head();
        let options = self.expand_options();

        let ctx = self.ctx;

        let branches: TokenStream2 = self
            .branches
            .iter()
            .map(|(method, branch)| {
                let method = method;

                quote! {
                    method if method == hyper::Method::#method => #branch,
                }
            })
            .collect();

        quote! {
            match #ctx.req.method() {
                #branches
                #head
                #options
                _ => Err(crate::StatusCodeError::method_not_allowed().into()),
            }
        }
    }
}

impl Parse for MethodRoot {
    fn parse(input: ParseStream) -> Result<Self> {
        let ctx = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        braced!(content in input);

        let mut branches = HashMap::new();
        while !content.is_empty() {
            let method = content.parse()?;
            content.parse::<Token![=>]>()?;
            let branch = content.parse()?;

            branches.insert(method, branch);

            // Parse optional ',' at the end of branch.
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(Self { ctx, branches })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum Method {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

impl Method {
    fn as_str(&self) -> &str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Patch => "PATCH",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
        }
    }
}

impl Parse for Method {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse::<Ident>()?;

        match ident.to_string().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PATCH" => Ok(Self::Patch),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "OPTIONS" => panic!("OPTIONS is not allowed"),
            "HEAD" => panic!("HEAD is not allowed"),
            "TRACE" => panic!("TRACE is not allowed"),
            "CONNECT" => panic!("CONNECT is not allowed"),
            _ => panic!("Unknown request method"),
        }
    }
}

impl ToTokens for Method {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = Ident::new(self.as_str(), Span::call_site());

        tokens.extend(quote! { #ident });
    }
}
