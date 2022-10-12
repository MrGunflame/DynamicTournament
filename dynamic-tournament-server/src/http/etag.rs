use std::fmt::{self, Display, Formatter};
use std::hash::Hasher;
use std::str::FromStr;

use hyper::header::HeaderValue;
use sha1::digest::Output;
use sha1::{Digest, Sha1};

/// An entity tag (etag) for a resource.
///
/// This is a cheaply created and copy-able hash internally.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Etag {
    buf: Output<Sha1>,
}

impl Etag {
    pub fn new<T>(value: &T) -> Self
    where
        T: ?Sized + HashEtag,
    {
        let mut hasher = EtagHasher(Sha1::new());
        value.hash(&mut hasher);
        let buf = hasher.0.finalize();

        Self { buf }
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
        for b in &self.buf {
            write!(f, "{:x}", *b)?;
        }

        Ok(())
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
        let mut buf: Output<Sha1> = Default::default();
        hex::decode_to_slice(s, &mut buf)?;

        Ok(Self { buf })
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
        H: Hasher;
}

#[derive(Debug)]
pub struct EtagHasher(Sha1);

impl Hasher for EtagHasher {
    fn write(&mut self, bytes: &[u8]) {
        Digest::update(&mut self.0, bytes);
    }

    fn finish(&self) -> u64 {
        0
    }
}
