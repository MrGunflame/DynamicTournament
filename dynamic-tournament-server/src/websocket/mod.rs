use crate::State;

use dynamic_tournament_api::tournament::{Bracket, BracketType, Team, Tournament};
use dynamic_tournament_api::websocket;
use dynamic_tournament_generator::{
    DoubleElimination, EntrantScore, EntrantSpot, SingleElimination,
};

use futures::SinkExt;
use futures::StreamExt;
use hyper::upgrade::Upgraded;
use parking_lot::lock_api::RwLockUpgradableReadGuard;
use parking_lot::RwLock;
use tokio::select;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::{self, CloseFrame, Role};
use tokio_tungstenite::WebSocketStream;

use std::borrow::Cow;
use std::ops::DerefMut;
use std::sync::Arc;
use std::time::Duration;

pub async fn handle(conn: Upgraded, state: State, id: u64) {
    let mut shutdown_rx = state.shutdown_rx.clone();

    let stream = WebSocketStream::from_raw_socket(conn, Role::Server, None).await;

    let tournament = state.get_tournament(id).await.unwrap().unwrap();
    let bracket = state.get_bracket(id).await.unwrap();

    let (mut sub_rx, bracket) = {
        let subscribers = state.subscribers.upgradable_read();

        match subscribers.get(&id) {
            Some(b) => (b.subscribe(), b.clone()),
            None => {
                let (bracket, rx) = match LiveBracket::new(tournament, bracket) {
                    Ok(v) => v,
                    Err(err) => {
                        log::error!("Failed to create new LiveBracket: {err}");
                        return;
                    }
                };

                let mut subscribers = RwLockUpgradableReadGuard::upgrade(subscribers);

                let b2 = bracket.clone();

                subscribers.insert(id, bracket);

                (rx, b2)
            }
        }
    };

    let (tx, mut rx) = mpsc::channel::<WebSocketMessage>(32);
    let (close_tx, close_rx) = oneshot::channel::<()>();

    let (mut sink, mut stream) = stream.split();

    // Reader
    let state2 = state.clone();
    tokio::task::spawn(async move {
        let mut is_authenticated = false;
        let mut shutdown_notify = None;

        loop {
            select! {
                msg = stream.next() => {
                    match msg {
                        Some(msg) => match msg {
                            Ok(msg) => match msg {
                                // Text is not supported. Close the connection immediately if a frame text is
                                // received.
                                protocol::Message::Text(_) => {
                                    log::debug!("Received a text frame from client");
                                    break;
                                }
                                protocol::Message::Binary(bytes) => {
                                    log::debug!("Received a binary frame from client");

                                    let msg = match websocket::Message::from_bytes(&bytes) {
                                        Ok(msg) => msg,
                                        Err(err) => {
                                            log::debug!("Failed to deserialize message: {:?}", err);
                                            break;
                                        }
                                    };

                                    match msg {
                                        websocket::Message::Reserved => (),
                                        websocket::Message::Authorize(string) => {
                                            if state.is_authenticated_string(&string) {
                                                is_authenticated = true;
                                            } else {
                                                break;
                                            }
                                        }
                                        websocket::Message::UpdateMatch { index, nodes } => {
                                            // Only update the bracket when the client is authenticated.
                                            // Otherwise we will just ignore the message.
                                            if is_authenticated {
                                                bracket.update(index.try_into().unwrap(), nodes);
                                                store_bracket(&bracket, &state2, id).await;
                                            }
                                        }
                                        websocket::Message::ResetMatch { index } => {
                                            // Only update the bracket when the client is authenticated.
                                            // Otherwise we will just ignore the message.
                                            if is_authenticated {
                                                bracket.reset(index);
                                                store_bracket(&bracket, &state2, id).await;
                                            }
                                        }
                                    }
                                }
                                protocol::Message::Ping(buf) => {
                                    let _ = tx.send(WebSocketMessage::Pong(buf)).await;
                                }
                                protocol::Message::Pong(_) => (),
                                protocol::Message::Close(_) => {
                                    // Closing handshake initialized from server.
                                    if shutdown_notify.is_some() {
                                        break;
                                    }

                                    let _ = tx.send(WebSocketMessage::Close).await;
                                    break;
                                }
                                protocol::Message::Frame(_) => unreachable!(),
                            },
                            Err(err) => {
                                log::warn!("Failed to read from stream: {:?}", err);
                                break;
                            }
                        },
                        None => break,
                    }
                }
                _ = shutdown_rx.changed() => {
                    log::debug!("Closing websocket connection due to server shutdown");

                    let _ = tx.send(WebSocketMessage::Close).await;

                    shutdown_notify = Some(shutdown_rx.borrow().clone().unwrap());
                }
            }
        }

        // Wait for the writer to close.
        let _ = close_rx.await;
        let _ = shutdown_notify.unwrap().send(true).await;
    });

    // Writer
    tokio::task::spawn(async move {
        // Interval timer for pings.
        let mut interval = tokio::time::interval(Duration::new(30, 0));
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            select! {
                _ = interval.tick() => {
                    log::debug!("Sending ping to client");

                    if let Err(err) = sink.send(protocol::Message::Ping(vec![0])).await {
                        log::warn!("Failed to send ping: {:?}", err);
                        break;
                    }
                }
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => match msg {
                            WebSocketMessage::Message(msg) => {
                                let bytes = msg.into_bytes();

                                if let Err(err) = sink.send(protocol::Message::Binary(bytes)).await {
                                    log::warn!("Failed to send frame: {:?}", err);
                                    break;
                                }
                            }
                            WebSocketMessage::Pong(buf) => {
                                log::debug!("Sending pong");

                                if let Err(err) = sink.send(protocol::Message::Pong(buf)).await {
                                    log::warn!("Failed to send frame: {:?}", err);
                                    break;
                                }
                            }
                            WebSocketMessage::Close => {
                                log::debug!("Closing websocket connection");

                                if let Err(err) = sink.send(close_normal()).await {
                                    log::warn!("Failed to send close frame: {:?}", err);
                                    break;
                                }
                            }
                        },
                        None => break,
                    }
                }
                // Listen to messages from the subscriber.
                msg = sub_rx.recv() => {
                    let msg = msg.unwrap();
                    let bytes = msg.into_bytes();

                    if let Err(err) = sink.send(protocol::Message::Binary(bytes)).await {
                        log::warn!("Failed to send frame: {:?}", err);
                        break;
                    }
                }
            }
        }

        // Always try to close the sink at the end.
        if let Err(err) = sink.close().await {
            log::warn!("Failed to close sink: {:?}", err);
        }

        let _ = close_tx.send(());
    });
}

#[derive(Debug)]
enum WebSocketMessage {
    #[allow(dead_code)]
    Message(websocket::Message),
    Pong(Vec<u8>),
    Close,
}

pub fn close_normal() -> protocol::Message {
    protocol::Message::Close(Some(CloseFrame {
        code: CloseCode::Normal,
        reason: Cow::Borrowed("CLOSE_NORMAL"),
    }))
}

#[derive(Clone, Debug)]
pub struct LiveBracket {
    inner: Arc<RwLock<InnerLiveBracket>>,
}

#[derive(Debug)]
struct InnerLiveBracket {
    bracket: TournamentBracket,
    // Note: This could be a spmc channel. tokio::sync::watch is not appropriate however since
    // more than just the recent value is required.
    tx: broadcast::Sender<websocket::Message>,
}

impl LiveBracket {
    pub fn new(
        tournament: Tournament,
        bracket: Option<Bracket>,
    ) -> Result<(Self, broadcast::Receiver<websocket::Message>), crate::Error> {
        let bracket = match tournament.bracket_type {
            BracketType::SingleElimination => {
                let bracket = match bracket {
                    Some(bracket) => {
                        let entrants = tournament.entrants.unwrap_teams().into();

                        SingleElimination::resume(entrants, bracket.0)?
                    }
                    None => SingleElimination::new(tournament.entrants.unwrap_teams().into_iter()),
                };

                TournamentBracket::SingleElimination(bracket)
            }
            BracketType::DoubleElimination => {
                let bracket = match bracket {
                    Some(bracket) => {
                        let entrants = tournament.entrants.unwrap_teams().into();

                        DoubleElimination::resume(entrants, bracket.0)?
                    }
                    None => DoubleElimination::new(tournament.entrants.unwrap_teams().into_iter()),
                };

                TournamentBracket::DoubleElimination(bracket)
            }
        };

        let (tx, rx) = broadcast::channel(8);

        let inner = Arc::new(RwLock::new(InnerLiveBracket { bracket, tx }));

        Ok((Self { inner }, rx))
    }

    /// Creates a new [`Receiver`] for updates of this `LiveBracket`.
    ///
    /// [`Receiver`]: broadcast::Receiver
    pub fn subscribe(&self) -> broadcast::Receiver<websocket::Message> {
        let inner = self.inner.read();

        inner.tx.subscribe()
    }

    /// Updates the match at `index` using the given `nodes`.
    pub fn update(&self, index: usize, nodes: [EntrantScore<u64>; 2]) {
        let mut inner = self.inner.write();

        match inner.bracket {
            TournamentBracket::SingleElimination(ref mut bracket) => {
                bracket.update_match(index, |m, res| {
                    let mut loser_index = None;

                    for (i, (entrant, node)) in m.entrants.iter_mut().zip(nodes).enumerate() {
                        if let EntrantSpot::Entrant(entrant) = entrant {
                            *entrant.deref_mut() = node;
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
            }
            TournamentBracket::DoubleElimination(ref mut bracket) => {
                bracket.update_match(index, |m, res| {
                    let mut loser_index = None;

                    for (i, (entrant, node)) in m.entrants.iter_mut().zip(nodes).enumerate() {
                        if let EntrantSpot::Entrant(entrant) = entrant {
                            *entrant.deref_mut() = node;
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
            }
        }

        let _ = inner.tx.send(websocket::Message::UpdateMatch {
            index: index.try_into().unwrap(),
            nodes,
        });
    }

    /// Resets the match at `index`.
    pub fn reset(&self, index: usize) {
        let mut inner = self.inner.write();

        match inner.bracket {
            TournamentBracket::SingleElimination(ref mut bracket) => {
                bracket.update_match(index, |_, res| {
                    res.reset_default();
                });
            }
            TournamentBracket::DoubleElimination(ref mut bracket) => {
                bracket.update_match(index, |_, res| {
                    res.reset_default();
                });
            }
        }

        let _ = inner.tx.send(websocket::Message::ResetMatch { index });
    }
}

#[derive(Clone, Debug)]
pub enum TournamentBracket {
    SingleElimination(SingleElimination<Team, EntrantScore<u64>>),
    DoubleElimination(DoubleElimination<Team, EntrantScore<u64>>),
}

pub async fn store_bracket(bracket: &LiveBracket, state: &State, id: u64) {
    let bracket = {
        let inner = bracket.inner.read();

        inner.bracket.clone()
    };

    state
        .update_bracket(
            id,
            match bracket {
                TournamentBracket::SingleElimination(b) => Bracket(b.into_matches()),
                TournamentBracket::DoubleElimination(b) => Bracket(b.into_matches()),
            },
        )
        .await
        .unwrap();
}
