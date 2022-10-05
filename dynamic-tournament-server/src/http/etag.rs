use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use crc32fast::Hasher;
use hyper::header::HeaderValue;

/// An entity tag (etag) for a resource.
///
/// This is a cheaply created and copy-able hash internally.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Etag {
    hash: u32,
}

impl Etag {
    pub fn new<T>(value: &T) -> Self
    where
        T: ?Sized + HashEtag,
    {
        let mut hasher = Hasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finalize();

        Self { hash }
    }

    pub fn from_fn<F>(f: F) -> Self
    where
        F: FnOnce(&mut Hasher),
    {
        let mut hasher = Hasher::new();
        f(&mut hasher);
        let hash = hasher.finalize();

        Self { hash }
    }

    #[inline]
    pub fn matches<T>(self, value: &T) -> bool
    where
        T: ?Sized + HashEtag,
    {
        self == Self::new(value)
    }
}

impl Display for Etag {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let bytes = self.hash.to_le_bytes();

        f.write_str(&hex::encode(bytes))
    }
}

impl From<Etag> for HeaderValue {
    fn from(etag: Etag) -> Self {
        let bytes = etag.to_string();

        // SAFETY: The `Etag` only contains alphanumeric characters.
        unsafe { HeaderValue::from_maybe_shared_unchecked(bytes) }
    }
}

impl FromStr for Etag {
    type Err = hex::FromHexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut buf = [0; 4];
        hex::decode_to_slice(s, &mut buf)?;

        Ok(Self {
            hash: u32::from_le_bytes(buf),
        })
    }
}

/// A special hash trait analogous [`Hash`] used for creating [`Etag`]s.
///
/// [`Hash`]: std::hash::Hash
pub trait HashEtag {
    /// Feeds this value into the given [`Hasher`].
    ///
    /// [`Hasher`]: std::hash::Hasher
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher;
}
