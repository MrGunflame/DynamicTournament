use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};

use chrono::Utc;
use dynamic_tournament_api::v3::id::{BracketId, EntrantId, EventId, SystemId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::Response;
use dynamic_tournament_api::v3::tournaments::log::{LogEvent, LogEventBody};
use dynamic_tournament_core::{
    tournament::{Tournament, TournamentKind},
    EntrantScore, EntrantSpot, Matches, System,
};
use futures::{ready, Stream};
use parking_lot::RwLock;
use tokio::sync::broadcast;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;

use crate::{store::Store, Error};

#[derive(Clone, Debug)]
pub struct LiveBracket {
    /// Id of the connected user.
    user_id: Option<u64>,
    inner: Arc<LiveBracketInner>,
}

impl LiveBracket {
    pub fn set_user_id(&mut self, id: u64) {
        self.user_id = Some(id);
    }

    pub fn update(&self, index: u64, nodes: [EntrantScore<u64>; 2]) {
        let mut bracket = self.inner.bracket.write();

        bracket.update_match(index.try_into().unwrap(), |m, res| {
            let mut loser_index = None;

            for (i, (entrant, node)) in m.entrants.iter_mut().zip(nodes).enumerate() {
                if let EntrantSpot::Entrant(entrant) = entrant {
                    entrant.data = node;
                }

                if node.winner {
                    res.winner_default(entrant);
                    loser_index = Some(match i {
                        0 => 1,
                        _ => 0,
                    });
                }
            }

            if let Some(loser_index) = loser_index {
                res.loser_default(&m.entrants[loser_index]);
            }
        });

        self.notify(BracketChange::UpdateMatch { index, nodes });

        let log_event = LogEvent {
            id: EventId(0),
            date: Utc::now(),
            author: self.user_id.unwrap_or(0),
            body: LogEventBody::UpdateMatch {
                bracket_id: self.inner.bracket_id,
                index,
                nodes,
            },
        };

        let bracket = self.clone();
        tokio::task::spawn(async move {
            if let Err(err) = bracket.log(log_event).await {
                log::error!("Failed to log event: {}", err);
            }

            if let Err(err) = bracket.store().await {
                log::error!("Failed to save bracket state: {}", err);
            }
        });
    }

    pub fn reset(&self, index: usize) {
        let mut bracket = self.inner.bracket.write();

        bracket.update_match(index, |_, res| {
            res.reset_default();
        });

        self.notify(BracketChange::ResetMatch { index });

        let log_event = LogEvent {
            id: EventId(0),
            date: Utc::now(),
            author: self.user_id.unwrap_or(0),
            body: LogEventBody::ResetMatch {
                bracket_id: self.inner.bracket_id,
                index: index as u64,
            },
        };

        let bracket = self.clone();
        tokio::task::spawn(async move {
            if let Err(err) = bracket.log(log_event).await {
                log::error!("Failed to log event: {}", err);
            }

            if let Err(err) = bracket.store().await {
                log::error!("Failed to save bracket state: {}", err);
            }
        });
    }

    fn notify(&self, event: BracketChange) {
        log::debug!(
            "Notify {} listeners for LiveBracket",
            self.inner.tx.receiver_count()
        );

        // Note that this operation can never fail. We keep a receiver ourselves.
        let _ = self.inner.tx.send(event);
    }

    pub fn matches(&self) -> Matches<EntrantScore<u64>> {
        let bracket = self.inner.bracket.read().clone();
        bracket.into_matches()
    }

    pub fn receiver(&self) -> broadcast::Receiver<BracketChange> {
        self.inner.tx.subscribe()
    }

    pub fn changed(&self) -> EventStream<'_> {
        let rx = self.receiver();

        EventStream {
            _bracket: self,
            state: BroadcastStream::new(rx),
            buf: None,
        }
    }

    pub async fn store(&self) -> Result<(), Error> {
        let matches = self.inner.bracket.read().clone().into_matches();

        self.inner
            .store
            .update_bracket_state(
                self.inner.tournament_id,
                self.inner.bracket_id,
                &Some(matches),
            )
            .await?;
        Ok(())
    }

    pub async fn log(&self, event: LogEvent) -> Result<(), Error> {
        self.inner
            .store
            .event_log(self.inner.tournament_id)
            .insert(&event)
            .await
    }
}

#[derive(Debug)]
pub struct LiveBracketInner {
    store: Store,
    tournament_id: TournamentId,
    bracket_id: BracketId,
    bracket: RwLock<Tournament<EntrantId, EntrantScore<u64>>>,
    tx: broadcast::Sender<BracketChange>,

    #[allow(clippy::type_complexity)]
    live_brackets: Arc<RwLock<HashMap<(TournamentId, BracketId), Weak<LiveBracketInner>>>>,
}

impl Drop for LiveBracketInner {
    fn drop(&mut self) {
        let mut bracket = self.live_brackets.write();
        bracket.remove(&(self.tournament_id, self.bracket_id));
    }
}

#[derive(Clone, Debug)]
pub struct LiveBrackets {
    store: Store,
    #[allow(clippy::type_complexity)]
    inner: Arc<RwLock<HashMap<(TournamentId, BracketId), Weak<LiveBracketInner>>>>,
}

impl LiveBrackets {
    #[inline]
    pub fn new(store: Store) -> Self {
        Self {
            store,
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Returns a new [`LiveBracket`] if is already registered in memory. This method won't try to
    /// create a new [`LiveBracket`] from an the data loaded from a store.
    #[inline]
    pub fn get_local(
        &self,
        tournament_id: TournamentId,
        bracket_id: BracketId,
    ) -> Option<LiveBracket> {
        let bracket = self.inner.read();
        let bracket = bracket.get(&(tournament_id, bracket_id))?;
        Some(LiveBracket {
            user_id: None,
            inner: bracket.clone().upgrade().unwrap(),
        })
    }

    /// Returns a new [`LiveBracket`] for the associated tournament and bracket id.
    pub async fn get(
        &self,
        tournament_id: TournamentId,
        bracket_id: BracketId,
    ) -> Result<LiveBracket, Error> {
        log::debug!(
            "Getting LiveBracket {{ tournament_id: {}, bracket_id: {}}}",
            tournament_id,
            bracket_id
        );

        if let Some(bracket) = self.get_local(tournament_id, bracket_id) {
            return Ok(bracket);
        }

        log::debug!("LiveBracket not found in map, fetching from store");

        let bracket = self
            .store
            .get_bracket(tournament_id, bracket_id)
            .await?
            .unwrap();

        let state = self
            .store
            .get_bracket_state(tournament_id, bracket_id)
            .await?;

        let kind = match bracket.system {
            SystemId(1) => TournamentKind::SingleElimination,
            SystemId(2) => TournamentKind::DoubleElimination,
            SystemId(3) => TournamentKind::RoundRobin,
            SystemId(4) => TournamentKind::Swiss,
            _ => unreachable!(),
        };

        let tournament = match state {
            Some(matches) => {
                Tournament::resume(kind, bracket.entrants.into(), matches, bracket.options).unwrap()
            }
            None => {
                let mut tournament = Tournament::new(kind, bracket.options);
                tournament.extend(bracket.entrants);
                tournament
            }
        };

        let (tx, _) = broadcast::channel(32);

        let bracket = LiveBracket {
            user_id: None,
            inner: Arc::new(LiveBracketInner {
                store: self.store.clone(),
                tournament_id,
                bracket_id,
                bracket: RwLock::new(tournament),
                tx,

                live_brackets: self.inner.clone(),
            }),
        };

        let mut inner = self.inner.write();
        inner.insert((tournament_id, bracket_id), Arc::downgrade(&bracket.inner));

        log::debug!("Created new LiveBracket");

        Ok(bracket)
    }
}

/// A [`Stream`] over all upcoming [`BracketChange`] events.
#[derive(Debug)]
pub struct EventStream<'a> {
    _bracket: &'a LiveBracket,
    state: BroadcastStream<BracketChange>,
    // Since `BroadcastStream::poll_next` doesn't register for future values we need to
    // manually poll after the poll is complete. If the value returns immediately we store
    // them in the buffer.
    buf: Option<Result<BracketChange, ChangeError>>,
}

impl<'a> Stream for EventStream<'a> {
    type Item = Result<BracketChange, ChangeError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        log::trace!("EventStream.poll_next");

        // We have an item stored in the buffer.
        if let Some(buf) = self.buf.take() {
            let state = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.state) };

            if let Poll::Ready(val) = state.poll_next(cx) {
                let val = match val {
                    Some(Ok(c)) => Ok(c),
                    Some(Err(BroadcastStreamRecvError::Lagged(_))) => Err(ChangeError::Lagged),
                    None => unreachable!(),
                };

                self.buf = Some(val);
            }

            return Poll::Ready(Some(buf));
        }

        let mut state = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.state) };

        let res = ready!(state.as_mut().poll_next(cx));

        if let Poll::Ready(val) = state.poll_next(cx) {
            let val = match val {
                Some(Ok(c)) => Ok(c),
                Some(Err(BroadcastStreamRecvError::Lagged(_))) => Err(ChangeError::Lagged),
                None => unreachable!(),
            };

            self.buf = Some(val);
        }

        match res {
            Some(Ok(c)) => Poll::Ready(Some(Ok(c))),
            Some(Err(BroadcastStreamRecvError::Lagged(_))) => {
                Poll::Ready(Some(Err(ChangeError::Lagged)))
            }
            // This is unreachable because we always keep a sender in `self._bracket`.
            None => unreachable!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChangeError {
    Lagged,
}

#[derive(Clone, Debug)]
pub enum BracketChange {
    UpdateMatch {
        index: u64,
        nodes: [EntrantScore<u64>; 2],
    },
    ResetMatch {
        index: usize,
    },
}

impl From<BracketChange> for Response {
    fn from(this: BracketChange) -> Self {
        match this {
            BracketChange::UpdateMatch { index, nodes } => Response::UpdateMatch { index, nodes },
            BracketChange::ResetMatch { index } => Response::ResetMatch {
                index: index as u64,
            },
        }
    }
}
