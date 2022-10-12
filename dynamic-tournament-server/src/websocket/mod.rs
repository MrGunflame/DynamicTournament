pub mod live_bracket;

use std::io::Cursor;
use std::mem;

use crate::State;

use dynamic_tournament_api::auth::Flags;
use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::{
    Decode, ErrorResponse, Request, Response,
};

use dynamic_tournament_api::v3::tournaments::log::Event;
use futures::{SinkExt, Stream, StreamExt};
use hyper::upgrade::Upgraded;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::select;
use tokio::time::{Interval, MissedTickBehavior, Sleep};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Role};
use tokio_tungstenite::WebSocketStream;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use live_bracket::LiveBracket;

#[cfg(feature = "metrics")]
use crate::metrics::GaugeGuard;

use self::live_bracket::EventStream;

pub async fn handle(
    conn: Upgraded,
    state: State,
    tournament_id: TournamentId,
    bracket_id: BracketId,
) {
    log::debug!("Accepting new websocket connection");

    // Update the active connections gauge.
    #[cfg(feature = "metrics")]
    let _metrics_guard = {
        state.metrics.websocket_connections_total.inc();

        let gauge = state.metrics.websocket_connections_current.clone();
        GaugeGuard::new(gauge)
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
            log::debug!("Server shutdown, closing websocket connection");
            conn.close().await;
        }
    }
}

/// A websocket connection.
///
/// Note that the connection does **not** handle shutdown requests itself.
///
/// # Implementation notes
///
/// The connection is implemented using a single future. It calls [`poll_read`], [`poll_write`]
/// and [`poll_tick`] to advance the internal state. When a `poll_*` modifies the internal state
/// it must install a waker on the new state. The future implementation for `Connection` will only
/// initialize the state and then forward any `poll` calls to the appropriate `poll_*` method.
///
/// [`poll_read`]: Self::poll_read
/// [`poll_write`]: Self::poll_write
/// [`poll_tick`]: Self::poll_tick
#[derive(Debug)]
pub struct Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + 'static,
{
    stream: WebSocketStream<S>,

    // The ConnectionState must outlive self.stream.
    state: ConnectionState<'static, WebSocketStream<S>>,
    ping_interval: Interval,

    // Has the server initialized a close event.
    close_state: Option<CloseState>,

    global_state: State,
    bracket: LiveBracket,
    changed: EventStream<'static>,

    /// Id of the connected user. This is `None` if the user didn't authenticate yet.
    client_user: Option<u64>,
}

impl<S> Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + 'static,
{
    /// Creates a new `Connection` using `stream` as the underlying websocket stream. The
    /// initial handshake should already have happened.
    pub fn new(stream: WebSocketStream<S>, state: State, bracket: LiveBracket) -> Self
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let mut ping_interval = tokio::time::interval(Duration::new(30, 0));
        ping_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        let changed = unsafe { mem::transmute(bracket.changed()) };

        Self {
            stream,
            state: ConnectionState::Init,
            ping_interval,
            close_state: None,
            global_state: state,
            bracket,
            changed,
            client_user: None,
        }
    }

    /// Initiates a graceful shutdown of the `Connection`. The connection should be continued to
    /// be polled until it completes.
    #[inline]
    fn close(&mut self) -> Close<'_, S> {
        Close { conn: self }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        // If there is a message in the buffer, complete it.
        if let ConnectionState::Write(_) = self.state {
            return self.poll_write(cx);
        }

        self.init_write_close(None);
        self.poll_write_close(cx)
    }

    /// Poll the reading half of the stream. This method returns `Poll::Ready` once the remote
    /// sender is closed.
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        log::trace!("Connection.poll_read");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Read(_)));

        // If we initialized a close handshake and the client didn't respond
        // in time we forcefully close the connection.
        if let Some(close_state) = &mut self.close_state {
            if close_state.poll_deadline(cx).is_ready() {
                return Poll::Ready(());
            }
        }

        loop {
            let fut = match &mut self.state {
                ConnectionState::Init => unreachable!(),
                ConnectionState::Read(fut) => Pin::new(fut),
                ConnectionState::Write(_) => unreachable!(),
                ConnectionState::WriteClose(_) => unreachable!(),
                ConnectionState::Closed => unreachable!(),
            };

            match fut.poll(cx) {
                Poll::Ready(msg) => {
                    if let Some(msg) = msg {
                        match msg {
                            Ok(Message::Text(_)) => {
                                log::debug!("Received a text frame from client, that's illegal!");
                            }
                            Ok(Message::Binary(buf)) => {
                                log::debug!("Received a binary frame from client");
                                log::debug!("Reading websocket frame: {:?}", buf);

                                let mut buf = Cursor::new(buf);
                                let req = match Request::decode(&mut buf) {
                                    Ok(req) => req,
                                    Err(err) => {
                                        log::debug!("Failed to decode request: {}", err);

                                        let resp = Response::Error(ErrorResponse::Proto);
                                        self.init_write(Message::Binary(resp.to_bytes()));
                                        return self.poll_write(cx);
                                    }
                                };

                                if let Some(resp) = self.handle_request(req) {
                                    let buf = resp.to_bytes();

                                    self.init_write(Message::Binary(buf));
                                    return self.poll_write(cx);
                                }
                            }
                            Ok(Message::Ping(buf)) => {
                                self.init_write(Message::Pong(buf));
                                return self.poll_write(cx);
                            }
                            // Ignore pongs.
                            Ok(Message::Pong(buf)) => {
                                log::debug!("Received pong with payload: {:?}", buf);
                                continue;
                            }
                            Ok(Message::Close(frame)) => {
                                // Server-side close.
                                if let Some(close_state) = &self.close_state {
                                    // Client confirmed server close frame.
                                    if close_state.frame.as_ref() == frame.as_ref() {
                                        return Poll::Ready(());
                                    }
                                }

                                // Client-side close. We respond with the same close frame.
                                self.init_write_close(frame);
                                return self.poll_write_close(cx);
                            }
                            // Cannot receive a raw frame.
                            Ok(Message::Frame(_)) => unreachable!(),
                            Err(err) => {
                                log::debug!("Failed to read from stream: {}", err);
                                return Poll::Ready(());
                            }
                        }

                        // Next item
                        self.init_read();
                    } else {
                        return Poll::Ready(());
                    }
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }

    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        log::trace!("Connection.poll_write");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Write(_)));

        let fut = match &mut self.state {
            ConnectionState::Init => unreachable!(),
            ConnectionState::Read(_) => unreachable!(),
            ConnectionState::Write(fut) => Pin::new(fut),
            ConnectionState::WriteClose(_) => unreachable!(),
            ConnectionState::Closed => unreachable!(),
        };

        match fut.poll(cx) {
            Poll::Ready(_) => {
                // Switch back to read mode.
                self.init_read();
                self.poll_read(cx)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_write_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        log::trace!("Connection.poll_write_close");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::WriteClose(_)));

        let fut = match &mut self.state {
            ConnectionState::WriteClose(fut) => Pin::new(fut),
            _ => unreachable!(),
        };

        match fut.poll(cx) {
            Poll::Ready(res) => {
                if let Err(err) = res {
                    log::debug!("Failed to write close message: {}", err);
                }

                self.init_read();
                self.poll_read(cx)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_tick(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        log::trace!("Connection.poll_tick");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Read(_)));

        match self.ping_interval.poll_tick(cx) {
            Poll::Ready(_) => {
                self.init_write(Message::Ping(vec![0]));

                self.poll_write(cx)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    /// Poll for changes from the live bracket.
    fn poll_changed(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        log::trace!("Connection.poll_changed");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Read(_)));

        let changed = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.changed) };

        match changed.poll_next(cx) {
            Poll::Ready(res) => {
                let buf = match res {
                    Some(Ok(change)) => Response::from(change).to_bytes(),
                    // Lagged
                    Some(Err(_)) => Response::Error(ErrorResponse::Lagged).to_bytes(),
                    None => unreachable!(),
                };

                self.init_write(Message::Binary(buf));
                self.poll_write(cx)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn init_read(&mut self) {
        let prev = self.state_str();

        let fut = self.stream.next();

        // Extend Next<'a> into Next<'static>.
        // SAFETY: This is safe since the Next<'static> is only held for as long
        // as the  stream.
        let fut = unsafe { mem::transmute(fut) };

        self.state = ConnectionState::Read(fut);

        log::trace!("Connection.state {} -> {}", prev, self.state_str());
    }

    fn init_write(&mut self, msg: Message) {
        let prev = self.state_str();

        let fut = self.stream.send(msg);

        // Extend Send<'a> into Send<'static>.
        // SAFETY: This is safe since the Send<'static> is only held for as long
        // as the  stream.
        let fut = unsafe { mem::transmute(fut) };

        self.state = ConnectionState::Write(fut);

        log::trace!("Connection.state {} -> {}", prev, self.state_str());
    }

    fn init_write_close(&mut self, frame: Option<CloseFrame<'static>>) {
        let prev = self.state_str();

        self.close_state = Some(CloseState::new(frame.clone()));

        let fut = self.stream.send(Message::Close(frame));

        // Extend Send<'a> into Send<'static>.
        // SAFETY: This is safe since the Send<'static> is only held for as long
        // as the  stream.
        let fut = unsafe { mem::transmute(fut) };

        self.state = ConnectionState::WriteClose(fut);

        log::trace!("Connection.state {} -> {}", prev, self.state_str());
    }

    fn handle_request(&mut self, req: Request) -> Option<Response> {
        match req {
            Request::Reserved => None,
            Request::Authorize(token) => match self.global_state.auth.validate_auth_token(&token) {
                // The token is valid but we still need to verify the flags.
                Ok(token) => {
                    if token.claims().flags.intersects(Flags::EDIT_SCORES) {
                        self.client_user = Some(token.claims().sub);
                        None
                    } else {
                        Some(Response::Error(ErrorResponse::Unauthorized))
                    }
                }
                Err(err) => {
                    log::debug!("Failed to validate token: {}", err);
                    Some(Response::Error(ErrorResponse::Unauthorized))
                }
            },
            Request::SyncState => {
                let matches = self.bracket.matches();
                Some(Response::SyncState(matches))
            }
            Request::UpdateMatch { index, nodes } => match self.client_user {
                Some(author) => {
                    self.bracket.update(index, nodes);

                    let eventlog = self.global_state.eventlog.clone();
                    let tournament_id = self.bracket.tournament_id();
                    let bracket_id = self.bracket.bracket_id();
                    tokio::task::spawn(async move {
                        eventlog
                            .send(
                                tournament_id,
                                author,
                                Event::UpdateMatch {
                                    bracket: bracket_id,
                                    index: index as usize,
                                    nodes: nodes.into(),
                                },
                            )
                            .await;
                    });

                    None
                }
                None => Some(Response::Error(ErrorResponse::Unauthorized)),
            },
            Request::ResetMatch { index } => match self.client_user {
                Some(author) => {
                    self.bracket.reset(index as usize);

                    let eventlog = self.global_state.eventlog.clone();
                    let tournament_id = self.bracket.tournament_id();
                    let bracket_id = self.bracket.bracket_id();
                    tokio::task::spawn(async move {
                        eventlog
                            .send(
                                tournament_id,
                                author,
                                Event::ResetMatch {
                                    bracket: bracket_id,
                                    index: index as usize,
                                },
                            )
                            .await;
                    });

                    None
                }
                None => Some(Response::Error(ErrorResponse::Unauthorized)),
            },
        }
    }

    fn state_str(&self) -> &'static str {
        match self.state {
            ConnectionState::Init => "Init",
            ConnectionState::Read(_) => "Read",
            ConnectionState::Write(_) => "Write",
            ConnectionState::WriteClose(_) => "WriteClose",
            ConnectionState::Closed => "Closed",
        }
    }
}

/// Await state of the connection.
#[derive(Debug)]
enum ConnectionState<'a, S> {
    Init,
    /// Waiting for incoming messages.
    Read(futures::stream::Next<'a, S>),
    Write(futures::sink::Send<'a, S, Message>),
    WriteClose(futures::sink::Send<'a, S, Message>),
    Closed,
}

impl<S> Future for Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        log::trace!("Connection.poll");

        loop {
            match self.as_mut().state {
                // First call to poll, initialize the reader.
                ConnectionState::Init => {
                    self.init_read();
                    continue;
                }
                // Try to read, check for bracket updates and then tick.
                ConnectionState::Read(_) => match self.as_mut().poll_read(cx) {
                    // When `poll_read` returns `Poll::Ready` the connection is done.
                    Poll::Ready(()) => {
                        let prev = self.state_str();
                        self.state = ConnectionState::Closed;
                        log::trace!("Connection.state {} -> {}", prev, self.state_str());

                        return Poll::Ready(());
                    }
                    Poll::Pending => match self.as_mut().poll_changed(cx) {
                        Poll::Ready(()) => {}
                        Poll::Pending => return self.as_mut().poll_tick(cx),
                    },
                },
                // Finish the write request, then call poll_read again.
                ConnectionState::Write(_) => match self.as_mut().poll_write(cx) {
                    // Buffer written, return to reading.
                    Poll::Ready(()) => {
                        continue;
                    }
                    Poll::Pending => return Poll::Pending,
                },
                ConnectionState::WriteClose(_) => return self.poll_write_close(cx),
                ConnectionState::Closed => return Poll::Ready(()),
            }
        }
    }
}

unsafe impl<S> Send for Connection<S> where S: AsyncRead + AsyncWrite + Unpin + Send {}

// Both `TcpStream` and `UnixStream` are Sync, unlike Upgraded for some reason.
unsafe impl<S> Sync for Connection<S> where S: AsyncRead + AsyncWrite + Unpin {}

#[derive(Debug)]
pub struct Close<'a, S>
where
    S: AsyncRead + AsyncWrite + Unpin + 'static,
{
    conn: &'a mut Connection<S>,
}

impl<'a, S> Future for Close<'a, S>
where
    S: AsyncRead + AsyncWrite + Unpin + 'static,
{
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let conn = unsafe { self.map_unchecked_mut(|this| this.conn) };
        conn.poll_close(cx)
    }
}

/// The progress in the closing handshake. This is only necessary when the server closes
/// the connection.
#[derive(Debug)]
struct CloseState {
    frame: Option<CloseFrame<'static>>,
    /// The client must have responded to the close within this deadline.
    deadline: Pin<Box<Sleep>>,
}

impl CloseState {
    fn new(frame: Option<CloseFrame<'static>>) -> Self {
        Self {
            frame,
            deadline: Box::pin(tokio::time::sleep(Duration::new(5, 0))),
        }
    }

    fn poll_deadline(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        self.deadline.as_mut().poll(cx)
    }
}
