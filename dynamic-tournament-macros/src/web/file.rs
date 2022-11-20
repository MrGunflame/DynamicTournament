use std::fs::File;
use std::io::Read;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use proc_macro2::Literal;
use syn::Lit;

/// A included asset file.
///
/// This is the equivalent to [`std::include_str`], but has additional capabilites.
#[derive(Clone, Debug)]
pub struct AssetFile {
    buf: Vec<u8>,
    format: Option<FileFormat>,
    is_stripped: bool,
}

impl AssetFile {
    pub fn new<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        let format = file_format(path.as_ref());

        let mut file = File::open(path).unwrap();

        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();

        Self {
            buf,
            format,
            is_stripped: false,
        }
    }

    pub fn to_str(&mut self) -> Lit {
        if !self.is_stripped {
            self.strip();
            self.is_stripped = true;
        }

        let string = std::str::from_utf8(&self.buf).unwrap();

        Lit::Verbatim(Literal::string(&string))
    }

    fn strip(&mut self) {
        match self.format {
            Some(FileFormat::Svg) => {
                strip_svg(&mut self.buf);
            }
            None => (),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum FileFormat {
    Svg,
}

fn file_format<P>(path: P) -> Option<FileFormat>
where
    P: AsRef<Path>,
{
    match path.as_ref().extension()?.as_bytes() {
        b"svg" => Some(FileFormat::Svg),
        _ => None,
    }
}

fn strip_svg(buf: &mut Vec<u8>) {
    let mut s = std::str::from_utf8(&buf).unwrap();

    // Remove comments
    while let Some(start) = s.find("<!--") {
        let mut end = s.find("-->").unwrap();
        end += b"-->".len();

        drop(s);
        buf.drain(start..end);
        s = std::str::from_utf8(&buf).unwrap();
    }
}
