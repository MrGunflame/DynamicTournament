use std::ops::Deref;
use std::sync::Arc;

use crate::auth::Authorization;
use crate::signal::ShutdownListener;
use crate::store::Store;
use crate::websocket::live_bracket::LiveBrackets;
use crate::Config;
use crate::LoginData;

use sqlx::MySqlPool;

#[cfg(feature = "metrics")]
use crate::metrics::Metrics;

#[derive(Clone, Debug)]
pub struct State(Arc<StateInner>);

impl State {
    pub fn new(config: Config, users: Vec<LoginData>) -> Self {
        let pool = MySqlPool::connect_lazy(&config.database.connect_string()).unwrap();
        let store = Store { pool };

        let auth = Authorization::new(config.authorization.alg);

        let live_brackets = LiveBrackets::new(store.clone());

        Self(Arc::new(StateInner {
            store,
            users,
            config,
            live_brackets,
            shutdown: Shutdown,
            auth,

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
    pub auth: Authorization,

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
}

#[derive(Clone, Debug)]
pub struct Shutdown;

impl Shutdown {
    pub fn listen(&self) -> ShutdownListener<'static> {
        ShutdownListener::new()
    }
}
