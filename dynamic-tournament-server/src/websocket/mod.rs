pub mod live_bracket;

use crate::State;

use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::Frame;

use futures::future::{join3, Join3};
use futures::stream::{SplitSink, SplitStream};
use futures::SinkExt;
use futures::StreamExt;
use hyper::upgrade::Upgraded;
use parking_lot::Mutex;
use tokio::select;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Role};
use tokio_tungstenite::WebSocketStream;

use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use live_bracket::LiveBracket;

#[cfg(feature = "metrics")]
use crate::metrics::Metrics;

pub async fn handle(
    conn: Upgraded,
    state: State,
    tournament_id: TournamentId,
    bracket_id: BracketId,
) {
    #[cfg(feature = "metrics")]
    let _metrics_guard = {
        let metrics = state.metrics.clone();

        metrics.websocket_connections_total.inc();
        metrics.websocket_connections_current.inc();

        struct MetricsGuard {
            metrics: Metrics,
        }

        impl Drop for MetricsGuard {
            fn drop(&mut self) {
                self.metrics.websocket_connections_current.dec();
            }
        }

        MetricsGuard { metrics }
    };

    let shutdown = state.shutdown.listen();

    let stream = WebSocketStream::from_raw_socket(conn, Role::Server, None).await;

    let bracket = state
        .live_brackets
        .get(tournament_id, bracket_id)
        .await
        .unwrap();

    let mut conn = Connection::new(stream, state, bracket);

    select! {
        _ = &mut conn => {}
        _ = shutdown => {
            conn.shutdown().await;
            log::debug!("Server shutdown, closing websocket connection");
            conn.await;
        }
    }
}

#[derive(Debug)]
enum WebSocketMessage {
    Message(Frame),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame<'static>>),
}

fn close_normal() -> WebSocketMessage {
    WebSocketMessage::Close(Some(CloseFrame {
        code: CloseCode::Normal,
        reason: Cow::Borrowed("CLOSE_NORMAL"),
    }))
}

/// A websocket connection.
///
/// Note that the connection does **not** handle shutdown requests itself.
#[derive(Debug)]
pub struct Connection {
    tasks: Join3<JoinHandle<()>, JoinHandle<()>, JoinHandle<()>>,
    tx: mpsc::Sender<WebSocketMessage>,
}

impl Connection {
    pub fn new(stream: WebSocketStream<Upgraded>, state: State, bracket: LiveBracket) -> Self {
        // Reader to writer channel.
        let (tx, rx) = mpsc::channel(32);

        let state = Arc::new(ConnectionState {
            shutdown: Mutex::new(false),
            state,
            is_authenticated: Mutex::new(false),
            bracket,
            tx,
        });

        let (sink, stream) = stream.split();

        // Reader task
        let reader = {
            let state = state.clone();
            tokio::task::spawn(async move {
                drive_reader(stream, state).await;
            })
        };

        // Writer task
        let writer = {
            let state = state.clone();
            tokio::task::spawn(async move {
                drive_writer(sink, rx, state).await;
            })
        };

        // Send pings periodically.
        let tx = state.tx.clone();
        let ping = tokio::task::spawn(async move {
            let mut interval = tokio::time::interval(Duration::new(30, 0));
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                select! {
                    _ = tx.closed() => {
                        break;
                    }
                    _ = interval.tick() => {
                        let _ = tx.send(WebSocketMessage::Ping(vec![0])).await;
                    }
                }
            }
        });

        let tx = state.tx.clone();

        Self {
            tasks: join3(reader, writer, ping),
            tx,
        }
    }

    /// Initiates a graceful shutdown of the `Connection`. The connection should be continued to
    /// be polled until it completes.
    pub async fn shutdown(&self) {
        let _ = self.tx.send(close_normal()).await;
    }
}

impl Future for Connection {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let tasks = unsafe { self.map_unchecked_mut(|this| &mut this.tasks) };

        match tasks.poll(cx) {
            Poll::Ready(_) => Poll::Ready(()),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[derive(Debug)]
pub struct ConnectionState {
    /// The shutdown flag indicates whether the reader should drop when reading a close frame. This
    /// is only necessary if the server initiates the shutdown. When the client initiates the
    /// shutdown the reader is dropped immediately, and the writer consecutively.
    shutdown: Mutex<bool>,
    state: State,
    is_authenticated: Mutex<bool>,
    bracket: LiveBracket,
    tx: mpsc::Sender<WebSocketMessage>,
}

async fn drive_reader(
    mut stream: SplitStream<WebSocketStream<Upgraded>>,
    state: Arc<ConnectionState>,
) {
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(msg) => match msg {
                Message::Text(_) => {
                    log::warn!("Received a text frame from client, that's illegal!");
                    return;
                }
                Message::Binary(buf) => {
                    log::debug!("Received a binary frame from client");
                    log::debug!("Reading websocket frame: {:?}", buf);

                    let msg = match Frame::from_bytes(&buf) {
                        Ok(msg) => msg,
                        Err(err) => {
                            log::debug!("Failed to deserialize message: {:?}", err);
                            return;
                        }
                    };

                    handle_frame(msg, state.clone()).await;
                }
                // Respond to ping frames with the same payload in the pong frame.
                Message::Ping(buf) => {
                    let _ = state.tx.send(WebSocketMessage::Pong(buf)).await;
                }
                Message::Pong(_) => {}
                Message::Close(frame) => {
                    log::debug!("Received close frame: {:?}", frame);
                    let _ = state.tx.send(WebSocketMessage::Close(frame)).await;
                    break;
                }
                // Can't receive a raw frame.
                Message::Frame(_) => unreachable!(),
            },
            Err(err) => {
                log::warn!("Failed to read from ws stream: {:?}", err);
                break;
            }
        }
    }

    log::debug!("Dropping websocket conn reader");
}

async fn drive_writer(
    mut sink: SplitSink<WebSocketStream<Upgraded>, Message>,
    mut rx: mpsc::Receiver<WebSocketMessage>,
    state: Arc<ConnectionState>,
) {
    while let Some(msg) = rx.recv().await {
        log::debug!("Writing websocket frame: {:?}", msg);

        match msg {
            WebSocketMessage::Message(msg) => {
                let buf = msg.to_bytes().unwrap();

                if let Err(err) = sink.send(Message::Binary(buf)).await {
                    log::warn!("Failed to send frame: {:?}", err);
                    return;
                }
            }
            WebSocketMessage::Ping(buf) => {
                log::debug!("Sending ping with payload: {:?}", buf);

                if let Err(err) = sink.send(Message::Ping(buf)).await {
                    log::warn!("Failed to send frame: {:?}", err);
                    return;
                }
            }
            WebSocketMessage::Pong(buf) => {
                log::debug!("Sending pong with payload: {:?}", buf);

                if let Err(err) = sink.send(Message::Pong(buf)).await {
                    log::warn!("Failed to send frame: {:?}", err);
                    return;
                }
            }
            WebSocketMessage::Close(frame) => {
                log::debug!("Sending close frame: {:?}", frame);

                *state.shutdown.lock() = true;

                let msg = Message::Close(frame);
                if let Err(err) = sink.send(msg).await {
                    log::warn!("Failed to send close frame: {:?}", err);
                }

                break;
            }
        }
    }

    if let Err(err) = sink.close().await {
        log::warn!("Failed to close sink: {:?}", err);
    }

    log::debug!("Dropping websocket conn writer");
}

async fn handle_frame(frame: Frame, state: Arc<ConnectionState>) {
    match frame {
        Frame::Reserved => (),
        Frame::Authorize(string) => {
            if state.state.is_authenticated_string(&string) {
                *state.is_authenticated.lock() = true;
            }
        }
        Frame::UpdateMatch { index, nodes } => {
            if *state.is_authenticated.lock() {
                state.bracket.update(index, nodes);
                state
                    .state
                    .live_brackets
                    .store(&state.bracket)
                    .await
                    .unwrap();

                // Broadcast the change.
                let _ = state.tx.send(WebSocketMessage::Message(frame)).await;
            }
        }
        Frame::ResetMatch { index } => {
            if *state.is_authenticated.lock() {
                state.bracket.reset(index);
                state
                    .state
                    .live_brackets
                    .store(&state.bracket)
                    .await
                    .unwrap();

                // Broadcast the change.
                let _ = state.tx.send(WebSocketMessage::Message(frame)).await;
            }
        }
        Frame::SyncMatchesRequest => {
            let matches = state.bracket.matches();

            let _ = state
                .tx
                .send(WebSocketMessage::Message(Frame::SyncMatchesResponse(
                    matches,
                )))
                .await;
        }
        Frame::SyncMatchesResponse(_) => {}
    }
}
