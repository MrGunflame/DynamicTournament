use std::ops::Deref;
use std::sync::Arc;

use crate::http;
use crate::signal::ShutdownListener;
use crate::store::Store;
use crate::websocket::live_bracket::LiveBrackets;
use crate::Config;
use crate::LoginData;

use dynamic_tournament_api::auth::Claims;
use jsonwebtoken::{DecodingKey, Validation};
use sqlx::MySqlPool;

#[cfg(feature = "metrics")]
use crate::metrics::Metrics;

#[derive(Clone, Debug)]
pub struct State(Arc<StateInner>);

impl State {
    pub fn new(config: Config, users: Vec<LoginData>) -> Self {
        let pool = MySqlPool::connect_lazy(&config.database.connect_string()).unwrap();
        let store = Store { pool };

        let live_brackets = LiveBrackets::new(store.clone());

        Self(Arc::new(StateInner {
            store,
            users,
            config,
            live_brackets,
            shutdown: Shutdown,

            #[cfg(feature = "metrics")]
            metrics: Metrics::default(),
        }))
    }
}

impl Deref for State {
    type Target = StateInner;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct StateInner {
    pub store: Store,
    users: Vec<LoginData>,
    pub config: Config,
    pub live_brackets: LiveBrackets,
    pub shutdown: Shutdown,

    #[cfg(feature = "metrics")]
    pub metrics: Metrics,
}

impl State {
    pub fn is_allowed(&self, data: &LoginData) -> bool {
        log::debug!("Trying to authenticate: {}", data.username);

        for user in &self.users {
            if user.username == data.username && user.password == data.password {
                return true;
            }
        }

        false
    }

    pub fn is_authenticated(&self, req: &http::Request) -> bool {
        let header = match req.headers().get("Authorization") {
            Some(header) => header.as_bytes(),
            None => return false,
        };

        let header = match header.as_ref().strip_prefix(b"Bearer ") {
            Some(header) => header,
            None => return false,
        };

        self.is_authenticated_string(header)
    }

    pub fn is_authenticated_string(&self, header: impl AsRef<[u8]>) -> bool {
        match String::from_utf8(header.as_ref().to_vec()) {
            Ok(s) => self.decode_token(&s).is_ok(),
            Err(err) => {
                log::info!("Failed to convert header to string: {:?}", err);
                false
            }
        }
    }

    pub fn decode_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let key = DecodingKey::from_secret(http::v2::auth::SECRET);
        let validation = Validation::new(self.config.authorization.algorithm);

        let data = jsonwebtoken::decode(token, &key, &validation)?;

        Ok(data.claims)
    }
}

#[derive(Clone, Debug)]
pub struct Shutdown;

impl Shutdown {
    pub fn listen(&self) -> ShutdownListener<'static> {
        ShutdownListener::new()
    }
}
