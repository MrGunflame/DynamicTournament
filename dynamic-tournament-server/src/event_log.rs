//! This module is about logging of tournament events, for process logging see `logger.rs`.

use dynamic_tournament_api::v3::{
    id::TournamentId,
    tournaments::log::{Event, LogEntry},
};
use tokio::sync::mpsc;

use crate::store::Store;

pub fn spawn(store: Store) -> EventWriter {
    let (tx, rx) = mpsc::channel(32);

    tokio::task::spawn(async move {
        let mut rx = EventReader { rx };

        while let Some((tournament_id, author, event)) = rx.recv().await {
            let entry = LogEntry {
                id: 0.into(),
                author,
                event,
            };

            match store.log(tournament_id).insert(&entry).await {
                Ok(()) => (),
                Err(err) => {
                    log::error!("Failed to log event: {}", err);
                }
            }
        }

        log::debug!("All EventWriters dropped, stopping tournament logging");
    });

    EventWriter { tx }
}

#[derive(Debug)]
pub struct EventReader {
    rx: mpsc::Receiver<(TournamentId, u64, Event)>,
}

impl EventReader {
    pub async fn recv(&mut self) -> Option<(TournamentId, u64, Event)> {
        self.rx.recv().await
    }
}

#[derive(Clone, Debug)]
pub struct EventWriter {
    tx: mpsc::Sender<(TournamentId, u64, Event)>,
}

impl EventWriter {
    pub async fn send(&self, tournament_id: TournamentId, author: u64, event: Event) {
        let _ = self.tx.send((tournament_id, author, event)).await;
    }
}
