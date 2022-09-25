mod load_asset;

pub use load_asset::load_asset;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

use std::path::Path;
use std::process::Command;

use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Result};

#[derive(Clone, Debug)]
pub(crate) struct AssetPath {
    path: LitStr,
}

impl AssetPath {
    pub fn absolute(&self) -> LitStr {
        let mut path = self.asset_root();

        path.push_str(&self.path.value());

        if !Path::new(&path).is_file() {
            panic!("Cannot find asset file: {}", path);
        }

        LitStr::new(&path, Span::call_site())
    }

    fn asset_root(&self) -> String {
        let mut cmd = Command::new("git")
            .arg("rev-parse")
            .arg("--show-toplevel")
            .output()
            .unwrap();

        assert!(cmd.status.success());

        // Truncate '\n'
        cmd.stdout.truncate(cmd.stdout.len() - 1);

        let mut path = String::from_utf8(cmd.stdout).unwrap();

        path.push_str("/assets");

        let asset_path = Path::new(&path);
        if !asset_path.is_dir() {
            panic!("Cannot find assets directory: {:?}", path);
        }

        path
    }
}

impl Parse for AssetPath {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;

        Ok(Self { path })
    }
}

pub fn include_asset(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as AssetPath).absolute();

    TokenStream::from(quote! {
        ::core::include_str!(#lit)
    })
}
