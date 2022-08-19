use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Weak};
use std::task::{Context, Poll};

use dynamic_tournament_api::v3::id::{BracketId, EntrantId, SystemId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::Response;
use dynamic_tournament_core::{
    tournament::{Tournament, TournamentKind},
    EntrantScore, EntrantSpot, Matches, System,
};
use futures::StreamExt;
use parking_lot::RwLock;
use tokio::sync::broadcast;
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
use tokio_stream::wrappers::BroadcastStream;

use crate::{store::Store, Error};

#[derive(Clone, Debug)]
pub struct LiveBracket {
    inner: Arc<LiveBracketInner>,
}

impl LiveBracket {
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
                        _ => 1,
                    });
                }
            }

            if let Some(loser_index) = loser_index {
                res.loser_default(&m.entrants[loser_index]);
            }
        });

        let _ = self
            .inner
            .tx
            .send(BracketChange::UpdateMatch { index, nodes });

        let bracket = self.clone();
        tokio::task::spawn(async move {
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

        let _ = self.inner.tx.send(BracketChange::ResetMatch { index });

        let bracket = self.clone();
        tokio::task::spawn(async move {
            if let Err(err) = bracket.store().await {
                log::error!("Failed to save bracket state: {}", err);
            }
        });
    }

    pub fn matches(&self) -> Matches<EntrantScore<u64>> {
        let bracket = self.inner.bracket.read().clone();
        bracket.into_matches()
    }

    pub fn receiver(&self) -> broadcast::Receiver<BracketChange> {
        self.inner.tx.subscribe()
    }

    pub fn changed(&self) -> Changed<'_> {
        let rx = self.receiver();

        Changed {
            _bracket: self,
            state: BroadcastStream::new(rx),
            next: None,
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

pub struct Changed<'a> {
    _bracket: &'a LiveBracket,
    // TODO: Get rid the Box here once the asyncsync has a strong-typed broadcast channel.
    state: BroadcastStream<BracketChange>,
    next: Option<futures::stream::Next<'static, BroadcastStream<BracketChange>>>,
}

impl<'a> Future for Changed<'a> {
    type Output = Result<BracketChange, ChangeError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.next.is_none() {
            let next = self.state.next();
            // Extend to lifetime of Next to 'static.
            // SAFETY: This is safe since the reference only exists for the same
            // lifetime as the owner.
            let next = unsafe { std::mem::transmute(next) };

            self.next = Some(next);
        }

        let next = unsafe { self.map_unchecked_mut(|this| this.next.as_mut().unwrap()) };

        match next.poll(cx) {
            Poll::Ready(res) => match res {
                Some(Ok(c)) => Poll::Ready(Ok(c)),
                Some(Err(BroadcastStreamRecvError::Lagged(_))) => {
                    Poll::Ready(Err(ChangeError::Lagged))
                }
                // This error cannot ever occur because we keep an instance to a sender
                // ourself.
                None => unreachable!(),
            },
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<'a> std::fmt::Debug for Changed<'a> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
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
