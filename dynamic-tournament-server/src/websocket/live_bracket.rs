use std::collections::HashMap;
use std::sync::Arc;

use dynamic_tournament_api::v3::{
    id::{BracketId, EntrantId, SystemId, TournamentId},
    tournaments::brackets::matches::Frame,
};
use dynamic_tournament_generator::{
    tournament::{Tournament, TournamentKind},
    EntrantScore, EntrantSpot, Matches, System,
};
use parking_lot::RwLock;
use tokio::sync::broadcast;

use crate::{store::Store, Error};

#[derive(Debug)]
pub struct LiveBracket {
    tournament_id: TournamentId,
    bracket_id: BracketId,
    bracket: Arc<RwLock<Tournament<EntrantId, EntrantScore<u64>>>>,
    tx: broadcast::Sender<Frame>,
    rx: broadcast::Receiver<Frame>,
}

impl LiveBracket {
    pub fn update(&self, index: u64, nodes: [EntrantScore<u64>; 2]) {
        let mut bracket = self.bracket.write();

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

        let _ = self.tx.send(Frame::UpdateMatch { index, nodes });
    }

    pub fn reset(&self, index: usize) {
        let mut bracket = self.bracket.write();

        bracket.update_match(index, |_, res| {
            res.reset_default();
        });

        let _ = self.tx.send(Frame::ResetMatch { index });
    }

    pub fn matches(&self) -> Matches<EntrantScore<u64>> {
        let bracket = self.bracket.read().clone();
        bracket.into_matches()
    }

    pub async fn changed(&mut self) -> Frame {
        // This can't fail since we keep a `broadcast::Sender` in self.
        self.rx.recv().await.unwrap()
    }
}

impl Clone for LiveBracket {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            tournament_id: self.tournament_id,
            bracket_id: self.bracket_id,
            bracket: self.bracket.clone(),
            tx: self.tx.clone(),
            rx: self.tx.subscribe(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LiveBrackets {
    store: Store,
    inner: Arc<RwLock<HashMap<(TournamentId, BracketId), LiveBracket>>>,
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
        Some(bracket.clone())
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
            return Ok(bracket.clone());
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

        let (tx, rx) = broadcast::channel(32);

        let bracket = LiveBracket {
            tournament_id,
            bracket_id,
            bracket: Arc::new(RwLock::new(tournament)),
            tx,
            rx,
        };

        let mut inner = self.inner.write();
        inner.insert((tournament_id, bracket_id), bracket.clone());

        log::debug!("Created new LiveBracket");

        Ok(bracket)
    }

    pub async fn store(&self, bracket: &LiveBracket) -> Result<(), Error> {
        let matches = bracket.bracket.read().clone().into_matches();

        self.store
            .update_bracket_state(bracket.tournament_id, bracket.bracket_id, &Some(matches))
            .await?;
        Ok(())
    }
}
