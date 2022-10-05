use std::path::Path;
use std::process::Command;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, LitStr};

pub fn load_asset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as AssetPath);
    TokenStream::from(input.expand())
}

struct AssetPath {
    path: LitStr,
}

impl AssetPath {
    fn expand(self) -> TokenStream2 {
        let mut cmd = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
            .unwrap();

        assert!(cmd.status.success());

        // Truncate '\n'
        cmd.stdout.truncate(cmd.stdout.len() - 1);

        let mut path = String::from_utf8(cmd.stdout).unwrap();
        let rel_start = path.len();

        path.push_str("/assets");

        let asset_path = Path::new(&path);
        if !asset_path.is_dir() {
            panic!("Cannot find assets directory: {:?}", path);
        }

        let mut asset_path = self.path.value();
        if asset_path.starts_with('/') {
            asset_path.remove(0);
        }

        path.push('/');
        path.push_str(&asset_path);

        if !Path::new(&path).is_file() {
            panic!("Cannot find asset file: {}", path);
        }

        let lit = LitStr::new(&path[rel_start..], Span::call_site());

        quote! {
            #lit
        }
    }
}

impl Parse for AssetPath {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;

        Ok(Self { path })
    }
}
