use self::{auth::AuthClient, systems::SystemsClient, tournaments::TournamentsClient};

pub mod auth;
pub mod id;
pub mod systems;
pub mod tournaments;

#[derive(Clone, Debug)]
pub struct Client<'a> {
    inner: &'a crate::Client,
}

impl<'a> Client<'a> {
    pub(crate) fn new(client: &'a crate::Client) -> Self {
        Self { inner: client }
    }

    pub fn auth(&self) -> AuthClient {
        AuthClient::new(self.inner)
    }

    pub fn systems(&self) -> SystemsClient {
        SystemsClient::new(self.inner)
    }

    pub fn tournaments(&self) -> TournamentsClient {
        TournamentsClient::new(self.inner)
    }
}
