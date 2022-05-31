use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

macro_rules! id {
    ($name:ident, $id:ty) => {
        #[derive(
            Copy,
            Clone,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        #[repr(transparent)]
        #[serde(transparent)]
        pub struct $name(pub $id);

        impl Display for $name {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl AsRef<$id> for $name {
            #[inline]
            fn as_ref(&self) -> &$id {
                &self.0
            }
        }

        impl PartialEq<$id> for $name {
            #[inline]
            fn eq(&self, other: &$id) -> bool {
                self.0 == *other
            }
        }

        impl From<$id> for $name {
            #[inline]
            fn from(id: $id) -> Self {
                Self(id)
            }
        }

        impl FromStr for $name {
            type Err = <$id as FromStr>::Err;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(s.parse::<$id>()?))
            }
        }
    };
}

id!(TournamentId, u64);
id!(RoleId, u64);
id!(SystemId, u64);
id!(EntrantId, u64);
