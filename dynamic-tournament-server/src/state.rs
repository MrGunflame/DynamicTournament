use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use crate::auth::Authorization;
use crate::limits::Limits;
use crate::signal::ShutdownListener;
use crate::store::Store;
use crate::websocket::live_bracket::LiveBrackets;
use crate::Config;

use sqlx::pool::PoolOptions;
use sqlx::MySqlPool;

#[cfg(feature = "metrics")]
use crate::metrics::Metrics;

#[derive(Clone, Debug)]
pub struct State(Arc<StateInner>);

impl State {
    pub fn new(config: Config) -> Self {
        let limits = Limits::new();

        // Acquire STDIN, STDOUT and STDERR. They are acquired forever.
        if limits.try_acquire_files(3).map(|fd| fd.forget()).is_none() {
            panic!("Failed to acquire fds for STDIN, STDOUT and STDERR");
        }

        // Database connections
        limits.try_acquire_files(8).map(|fd| fd.forget()).unwrap();

        let pool: MySqlPool = PoolOptions::new()
            .max_connections(0)
            .max_connections(8)
            .max_lifetime(Duration::new(3600, 0))
            .idle_timeout(Duration::new(60, 0))
            .connect_lazy(&config.database.connect_string())
            .unwrap();

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

            #[cfg(feature = "limits")]
            limits,
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

#[derive(Debug)]
pub struct StateInner {
    pub store: Store,
    pub config: Config,
    pub live_brackets: LiveBrackets,
    pub shutdown: Shutdown,
    pub auth: Authorization,

    #[cfg(feature = "metrics")]
    pub metrics: Metrics,

    #[cfg(feature = "limits")]
    pub limits: Limits,
}

#[derive(Clone, Debug)]
pub struct Shutdown;

impl Shutdown {
    pub fn listen(&self) -> ShutdownListener<'static> {
        ShutdownListener::new()
    }
}
