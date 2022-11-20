mod file;
mod load_asset;

pub use load_asset::load_asset;
use proc_macro::TokenStream;
use quote::quote;

use std::path::Path;
use std::process::Command;

use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Result};

use self::file::AssetFile;

#[derive(Clone, Debug)]
pub(crate) struct AssetPath {
    path: LitStr,
}

impl AssetPath {
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

    pub fn path(&self) -> String {
        let mut path = self.asset_root();
        path.push_str(&self.path.value());
        path
    }
}

impl Parse for AssetPath {
    fn parse(input: ParseStream) -> Result<Self> {
        let path = input.parse()?;

        Ok(Self { path })
    }
}

pub fn include_asset_str(input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(input as AssetPath).path();

    let lit = AssetFile::new(path).to_str();

    TokenStream::from(quote! {
        #lit
    })
}
