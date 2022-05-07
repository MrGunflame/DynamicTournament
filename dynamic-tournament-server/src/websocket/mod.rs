use dynamic_tournament_api::tournament::{Bracket, Team, Tournament, TournamentId};
use dynamic_tournament_generator::{
    DoubleElimination, EntrantScore, EntrantSpot, SingleElimination,
};
use futures::{future, pin_mut, ready, FutureExt, Sink, SinkExt, Stream};
use hyper::upgrade::Upgraded;
use parking_lot::RwLock;
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::protocol::{self};
use tokio_tungstenite::WebSocketStream;

use dynamic_tournament_api::websocket;

use futures::{StreamExt, TryStreamExt};
use parking_lot::lock_api::RwLockUpgradableReadGuard;

use std::ops::DerefMut;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::State;

pub async fn handle(conn: Upgraded, state: State, id: u64) {
    // let stream = tokio_tungstenite::accept_async(conn).await.unwrap();
    let stream = WebSocket::from_raw_socket(conn, protocol::Role::Server, None).await;

    let tournament = state.get_tournament(id).await.unwrap().unwrap();
    let bracket = state.get_bracket(id).await.unwrap();

    let (mut sub_rx, bracket) = {
        let subscribers = state.subscribers.upgradable_read();

        match subscribers.get(&id) {
            Some(b) => (b.tx.subscribe(), b.clone()),
            None => {
                let (bracket, rx) = LiveBracket::load(tournament, bracket);

                let mut subscribers = RwLockUpgradableReadGuard::upgrade(subscribers);

                let b2 = bracket.clone();

                subscribers.insert(id, bracket);

                (rx, b2)
            }
        }
    };

    let (tx, mut rx) = mpsc::channel::<websocket::Message>(32);

    let (mut sink, mut stream) = stream.split();

    // Reader
    let state2 = state.clone();
    tokio::task::spawn(async move {
        let mut is_authenticated = false;

        while let Some(msg) = stream.next().await {
            let msg = msg.unwrap();

            match msg {
                Message { inner } => {
                    match inner {
                        protocol::Message::Binary(buf) => {
                            log::debug!("Got {:?} ({} bytes)", buf, buf.len());

                            let msg = match websocket::Message::from_bytes(&buf) {
                                Ok(msg) => msg,
                                Err(err) => {
                                    log::debug!("Message deserialization failed: {:?}", err);

                                    // Close the connection.
                                    let _ = tx.send(websocket::Message::Close).await;
                                    return;
                                }
                            };

                            match msg {
                                websocket::Message::Authorize(s) => {
                                    if state.is_authenticated_string(&s) {
                                        is_authenticated = true;
                                    } else {
                                        let _ = tx.send(websocket::Message::Close).await;
                                        return;
                                    }
                                }
                                websocket::Message::Close => {
                                    let _ = tx.send(websocket::Message::Close).await;
                                    return;
                                }
                                msg => {
                                    // Only update the bracket when the client is authenticated.
                                    // Otherwise we will just ignore the message.
                                    if is_authenticated {
                                        bracket.update(&state2, msg).await;
                                    }
                                }
                            }

                            #[cfg(debug_assertions)]
                            if !is_authenticated {
                                log::debug!("Client is not authenticated: skipping");
                            }
                        }
                        protocol::Message::Pong(_) => {}
                        // Unexpected packet, close the connection.
                        _ => {
                            let _ = tx.send(websocket::Message::Close).await;
                            return;
                        }
                    }
                }
            }
        }
    });

    // Writer
    tokio::task::spawn(async move {
        loop {
            tokio::select! {
                _ = tokio::time::sleep(std::time::Duration::new(30, 0)) => {
                    log::debug!("Sending ping to client");
                    sink.send(Message::ping(vec![0])).await.unwrap();
                }
                // Listen to message from the reader half.
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => match msg {
                            websocket::Message::Close => {
                                match sink.close().await {
                                    Err(err) => log::warn!("Failed to close sink: {:?}", err),
                                    _ => (),
                                }
                                return;
                            },
                            _ => unreachable!(),
                        }
                        None => {
                            match sink.close().await {
                                Err(err) => log::warn!("Failed to close sink: {:?}", err),
                                _ => (),
                            }
                            return;
                        }
                    }
                }
                // Listen to messages from the subscriber.
                msg = sub_rx.recv() => {
                    let msg = msg.unwrap();
                    let bytes = msg.into_bytes();

                    sink.send(Message::binary(bytes)).await.unwrap();

                    if let websocket::Message::Close = msg {
                        match sink.close().await {
                            Err(err) => log::warn!("Failed to close sink: {:?}", err),
                            _ => (),
                        }

                        return;
                    }
                }
            }
        }
    });
}

pub struct WebSocket {
    inner: WebSocketStream<hyper::upgrade::Upgraded>,
}

impl WebSocket {
    pub async fn from_raw_socket(
        upgraded: Upgraded,
        role: protocol::Role,
        config: Option<protocol::WebSocketConfig>,
    ) -> Self {
        WebSocketStream::from_raw_socket(upgraded, role, config)
            .map(|inner| WebSocket { inner })
            .await
    }

    pub async fn close(mut self) {
        future::poll_fn(|cx| Pin::new(&mut self).poll_close(cx))
            .await
            .unwrap();
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, crate::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match ready!(Pin::new(&mut self.inner).poll_next(cx)) {
            Some(Ok(item)) => Poll::Ready(Some(Ok(Message { inner: item }))),
            Some(Err(e)) => {
                log::debug!("websocket poll error: {}", e);
                Poll::Ready(panic!("{}", e))
            }
            None => {
                log::trace!("websocket closed");
                Poll::Ready(None)
            }
        }
    }
}

impl Sink<Message> for WebSocket {
    type Error = crate::Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match ready!(Pin::new(&mut self.inner).poll_ready(cx)) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(panic!("{}", e)),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        match Pin::new(&mut self.inner).start_send(item.inner) {
            Ok(()) => Ok(()),
            Err(e) => {
                log::debug!("websocket start_send error: {}", e);
                Err(panic!("{}", e))
            }
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match ready!(Pin::new(&mut self.inner).poll_flush(cx)) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(e) => Poll::Ready(panic!("{}", e)),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match ready!(Pin::new(&mut self.inner).poll_close(cx)) {
            Ok(()) => Poll::Ready(Ok(())),
            Err(err) => {
                log::debug!("websocket close error: {}", err);
                Poll::Ready(Err(panic!("{}", err)))
            }
        }
    }
}

pub struct Message {
    inner: protocol::Message,
}

impl Message {
    pub fn binary<T>(msg: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        Self {
            inner: protocol::Message::binary(msg),
        }
    }

    pub fn ping(msg: Vec<u8>) -> Self {
        Self {
            inner: protocol::Message::Ping(msg),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LiveBracket {
    id: TournamentId,
    bracket: Arc<RwLock<TournamentBracket>>,
    tx: broadcast::Sender<websocket::Message>,
}

impl LiveBracket {
    pub fn load(
        tournament: Tournament,
        bracket: Option<Bracket>,
    ) -> (Self, broadcast::Receiver<websocket::Message>) {
        let id = tournament.id;

        let bracket = match tournament.bracket_type {
            dynamic_tournament_api::tournament::BracketType::SingleElimination => {
                TournamentBracket::SingleElimination(match bracket {
                    Some(bracket) => SingleElimination::resume(
                        tournament.entrants.unwrap_teams().into(),
                        bracket.0,
                    )
                    .unwrap(),
                    None => SingleElimination::new(tournament.entrants.unwrap_teams().into_iter()),
                })
            }
            _ => unimplemented!(),
        };

        let (tx, rx) = broadcast::channel(32);

        (
            Self {
                id,
                bracket: Arc::new(RwLock::new(bracket)),
                tx,
            },
            rx,
        )
    }

    pub async fn update(&self, state: &State, msg: websocket::Message) {
        let bracket = {
            let mut bracket = self.bracket.write();

            match msg {
                websocket::Message::UpdateMatch { index, nodes } => {
                    let (index, nodes) = (index.try_into().unwrap(), nodes);

                    match *bracket {
                        TournamentBracket::SingleElimination(ref mut b) => {
                            b.update_match(index, |m, res| {
                                let mut has_winner = false;

                                for (entrant, node) in m.entrants.iter_mut().zip(nodes.into_iter())
                                {
                                    if let EntrantSpot::Entrant(entrant) = entrant {
                                        *entrant.deref_mut() = node;
                                    }

                                    if node.winner {
                                        res.winner_default(entrant);
                                        has_winner = true;
                                        continue;
                                    }

                                    if has_winner {
                                        res.loser_default(entrant);
                                        break;
                                    }
                                }
                            });
                        }
                        TournamentBracket::DoubleElimination(ref mut b) => {
                            b.update_match(index, |m, res| {
                                let mut has_winner = false;

                                for (entrant, node) in m.entrants.iter_mut().zip(nodes.into_iter())
                                {
                                    if let EntrantSpot::Entrant(entrant) = entrant {
                                        *entrant.deref_mut() = node;
                                    }

                                    if node.winner {
                                        res.winner_default(entrant);
                                        has_winner = true;
                                        continue;
                                    }

                                    if has_winner {
                                        res.loser_default(entrant);
                                        break;
                                    }
                                }
                            });
                        }
                    }
                }
                websocket::Message::ResetMatch { index } => {
                    let index = index.try_into().unwrap();

                    match *bracket {
                        TournamentBracket::SingleElimination(ref mut b) => {
                            b.update_match(index, |_, res| {
                                res.reset_default();
                            });
                        }
                        TournamentBracket::DoubleElimination(ref mut b) => {
                            b.update_match(index, |_, res| {
                                res.reset_default();
                            });
                        }
                    }
                }
                _ => unreachable!(),
            }

            bracket.clone()
        };

        state
            .update_bracket(
                self.id.0,
                match bracket {
                    TournamentBracket::SingleElimination(b) => Bracket(b.matches().clone()),
                    TournamentBracket::DoubleElimination(b) => Bracket(b.matches().clone()),
                },
            )
            .await
            .unwrap();

        self.tx.send(msg).unwrap();
    }
}

#[derive(Clone, Debug)]
pub enum TournamentBracket {
    SingleElimination(SingleElimination<Team, EntrantScore<u64>>),
    DoubleElimination(DoubleElimination<Team, EntrantScore<u64>>),
}
