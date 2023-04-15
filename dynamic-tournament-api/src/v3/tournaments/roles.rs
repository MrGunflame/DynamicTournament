use serde::{Deserialize, Serialize};

use crate::{
    v3::id::{RoleId, TournamentId},
    Client, Result,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Role {
    #[cfg_attr(feature = "server", serde(skip_deserializing))]
    pub id: RoleId,
    pub name: String,
}

pub struct RolesClient<'a> {
    client: &'a Client,
    tournament_id: TournamentId,
}

impl<'a> RolesClient<'a> {
    pub(crate) fn new(client: &'a Client, tournament_id: TournamentId) -> Self {
        Self {
            client,
            tournament_id,
        }
    }

    pub async fn list(&self) -> Result<Vec<Role>> {
        let uri = format!("/v3/tournaments/{}/roles", self.tournament_id);

        let req = self.client.request().get().uri(&uri).build();

        let resp = self.client.send(req).await?;

        resp.json().await
    }

    pub async fn get(&self, id: RoleId) -> Result<Role> {
        let uri = format!("/v3/tournaments/{}/roles/{}", self.tournament_id, id);

        let req = self.client.request().get().uri(&uri).build();

        self.client.send(req).await?.json().await
    }

    pub async fn create(&self, role: &Role) -> Result<Role> {
        let uri = format!("/v3/tournaments/{}/roles", self.tournament_id);

        let req = self.client.request().post().uri(&uri).body(role).build();

        self.client.send(req).await?.json().await
    }
}
