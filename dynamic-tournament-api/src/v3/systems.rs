use crate::{Client, Result};

use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

/// A unique identifier for a [`System`].
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct SystemId(pub u64);

impl Display for SystemId {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// A `System` defines the behavoir of a tournament bracket.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct System {
    pub id: SystemId,
    pub name: String,
}

#[derive(Copy, Clone)]
pub struct SystemsClient<'a> {
    client: &'a Client,
}

impl<'a> SystemsClient<'a> {
    /// Returns a list of all [`System`]s.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn list(&self) -> Result<Vec<System>> {
        let req = self.client.request().uri("/v2/systems").build();

        self.client.send(req).await?.json().await
    }

    /// Returns the [`System`] with the given `id`.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn get(&self, id: SystemId) -> Result<System> {
        let req = self
            .client
            .request()
            .uri(&format!("/v2/systems/{}", id))
            .build();

        self.client.send(req).await?.json().await
    }
}
