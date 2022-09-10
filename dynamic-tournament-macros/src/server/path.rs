use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{braced, parse_macro_input, Expr, Ident, Result, Token, Type};

pub fn path(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as PathInput);
    TokenStream::from(input.expand())
}

struct PathInput {
    uri: Expr,
    branches: Vec<(Path, Expr)>,
}

impl PathInput {
    fn expand(self) -> TokenStream2 {
        let branches: TokenStream2 = self
            .branches
            .iter()
            .map(|(path, branch)| match path {
                Path::Empty => quote! {
                    else if path == None {
                        ret = { #branch };
                    }
                },
                Path::Match(lit) => quote! {
                    else if path == Some(#lit) {
                        ret = { #branch };
                    }
                },
                Path::Parse(ident, type_hint) => match type_hint {
                    Some(type_hint) => {
                        quote! {
                            else if {
                                use ::core::str::FromStr;

                                match path {
                                    Some(path) => match <#type_hint as FromStr>::from_str(&path) {
                                        Ok(val) => {
                                            let #ident: #type_hint = val;

                                            ret = { #branch };

                                            true
                                        }
                                        Err(_) => {
                                            // This assignment isn't actually necessary since we always
                                            // return `false` from this branch, jumping to the catch-all
                                            // branch. This is only required because the compilter cannot
                                            // know that we always return `false`.
                                            ret = Err(crate::StatusCodeError::not_found().into());

                                            false
                                        }
                                    }
                                    None => {
                                        // This assignment isn't actually necessary since we always
                                        // return `false` from this branch, jumping to the catch-all
                                        // branch. This is only required because the compilter cannot
                                        // know that we always return `false`.
                                        ret = Err(crate::StatusCodeError::not_found().into());

                                        false
                                    }
                                }
                            } {},
                        }
                    }
                    None => {
                        quote! {
                            else if {
                                use ::core::str::FromStr;

                                match path {
                                    Some(path) => match FromStr::from_str(&path) {
                                        Ok(val) => {
                                            let #ident = val;

                                            ret = { #branch };

                                            true
                                        }
                                        Err(_) => {
                                            // This assignment isn't actually necessary since we always
                                            // return `false` from this branch, jumping to the catch-all
                                            // branch. This is only required because the compilter cannot
                                            // know that we always return `false`.
                                            ret = Err(crate::StatusCodeError::not_found().into());

                                            false
                                        },
                                    }
                                    None => {
                                        // This assignment isn't actually necessary since we always
                                        // return `false` from this branch, jumping to the catch-all
                                        // branch. This is only required because the compilter cannot
                                        // know that we always return `false`.
                                        ret = Err(crate::StatusCodeError::not_found().into());

                                        false
                                    },
                                }
                            } {}
                        }
                    }
                },
            })
            .collect();

        let uri = self.uri;

        quote! {
            {
                let path = #uri.take_str();
                let mut ret;

                // Use an always-false if statement, so we don't need special treatment for the
                // first branch and can always use `if else`.
                if false {
                    unreachable!();
                }
                #branches
                else {
                    ret = Err(crate::StatusCodeError::not_found().into());
                }

                ret
            }
        }
    }
}

impl Parse for PathInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let uri = input.parse()?;
        input.parse::<Token![,]>()?;

        let content;
        braced!(content in input);

        let mut branches = Vec::new();
        while !content.is_empty() {
            let path = content.parse()?;
            content.parse::<Token![=>]>()?;
            let branch = content.parse()?;

            branches.push((path, branch));

            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }

        Ok(Self { uri, branches })
    }
}

enum Path {
    Empty,
    Match(Expr),
    Parse(Ident, Option<Type>),
}

impl Parse for Path {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Token![@]) {
            input.parse::<Token![@]>()?;
            return Ok(Self::Empty);
        }

        if input.peek(Ident) {
            let ident = input.parse()?;

            let mut type_hint = None;
            if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;

                type_hint = Some(input.parse()?);
            }

            return Ok(Self::Parse(ident, type_hint));
        }

        let path = input.parse()?;
        Ok(Self::Match(path))
    }
}
