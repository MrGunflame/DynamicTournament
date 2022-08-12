use std::ops::Deref;
use std::sync::Arc;

use crate::auth::Authorization;
use crate::signal::ShutdownListener;
use crate::store::Store;
use crate::websocket::live_bracket::LiveBrackets;
use crate::Config;

use sqlx::MySqlPool;

#[cfg(feature = "metrics")]
use crate::metrics::Metrics;

#[derive(Clone, Debug)]
pub struct State(Arc<StateInner>);

impl State {
    pub fn new(config: Config) -> Self {
        let pool = MySqlPool::connect_lazy(&config.database.connect_string()).unwrap();
        let store = Store {
            pool,
            table_prefix: config.database.prefix.clone(),
        };

        let auth = Authorization::new(config.authorization.alg);

        let live_brackets = LiveBrackets::new(store.clone());

        Self(Arc::new(StateInner {
            store,
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
    pub config: Config,
    pub live_brackets: LiveBrackets,
    pub shutdown: Shutdown,
    pub auth: Authorization,

    #[cfg(feature = "metrics")]
    pub metrics: Metrics,
}

#[derive(Clone, Debug)]
pub struct Shutdown;

impl Shutdown {
    pub fn listen(&self) -> ShutdownListener<'static> {
        ShutdownListener::new()
    }
}
