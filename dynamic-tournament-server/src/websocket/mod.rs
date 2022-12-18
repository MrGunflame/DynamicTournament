pub mod live_bracket;

use std::io::Cursor;
use std::mem;

use crate::State;

use dynamic_tournament_api::auth::Flags;
use dynamic_tournament_api::v3::id::{BracketId, TournamentId};
use dynamic_tournament_api::v3::tournaments::brackets::matches::{
    Decode, ErrorResponse, Request, Response,
};

use futures::{Sink, Stream};
use hyper::upgrade::Upgraded;
use pin_project::pin_project;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::{Interval, MissedTickBehavior, Sleep};
use tokio::{select, time};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::tungstenite::protocol::{CloseFrame, Role};
use tokio_tungstenite::WebSocketStream;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use live_bracket::LiveBracket;

pub type Result<T> = std::result::Result<T, tokio_tungstenite::tungstenite::Error>;

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
            let _ = conn.close(None);
            let _ = conn.await;
        }
    }

    log::trace!("WebSocketStream closed");
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
#[pin_project]
pub struct Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + 'static,
{
    #[pin]
    stream: WebSocketStream<S>,

    // The ConnectionState must outlive self.stream.
    state: ConnectionState,
    ping_interval: Interval,
    pong_timeout: Option<Sleep>,

    // Has the server initialized a close event.
    close_frame: Option<Option<CloseFrame<'static>>>,

    global_state: State,
    bracket: LiveBracket,
    #[pin]
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
            state: ConnectionState::Read,
            ping_interval,
            pong_timeout: None,
            close_frame: None,
            global_state: state,
            bracket,
            changed,
            client_user: None,
        }
    }

    /// Transition the `Connection` into a closing state. Returns `Poll::Ready(())` immediately.
    #[inline]
    fn close(&mut self, frame: Option<CloseFrame<'static>>) -> Poll<Result<()>> {
        self.state = ConnectionState::Close(Message::Close(frame));
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        log::trace!("Connection.poll_close");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Close(_)));

        let mut this = self.project();

        match this.stream.as_mut().poll_ready(cx) {
            Poll::Ready(Ok(())) => (),
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            Poll::Pending => return Poll::Pending,
        }

        let msg = match this.state {
            ConnectionState::Close(msg) => msg.clone(),
            _ => unreachable!(),
        };

        this.stream.start_send(msg)?;

        *this.state = ConnectionState::Closed;
        Poll::Ready(Ok(()))
    }

    /// Poll the reading half of the stream. This method returns `Poll::Ready` once the remote
    /// sender is closed.
    ///
    /// `poll_read` always follows these steps in order:
    /// 1. Read incoming frames from the remote peer.
    /// 2. Check if the connection is timed out (and close it necessary).
    /// 3. Check for bracket state changes.
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        log::trace!("Connection.poll_read");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Read));

        let mut this = self.as_mut().project();

        if let Poll::Ready(msg) = this.stream.as_mut().poll_next(cx) {
            let msg = match msg {
                Some(Ok(msg)) => msg,
                Some(Err(err)) => return Poll::Ready(Err(err)),
                // The remote peer closed the stream.
                None => return self.close(None),
            };

            match msg {
                Message::Text(_) => {
                    log::debug!("Received a text frame from client, that's illegal!");
                }
                Message::Binary(buf) => {
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
                Message::Ping(buf) => {
                    self.init_write(Message::Pong(buf));
                    return self.poll_write(cx);
                }
                // Ignore pongs.
                Message::Pong(buf) => {
                    self.pong_timeout = None;
                    log::debug!("Received pong with payload: {:?}", buf);
                }
                Message::Close(frame) => {
                    // Server-side close.
                    if let Some(close_frame) = &self.close_frame {
                        // Client confirmed server close frame.
                        if close_frame.as_ref() == frame.as_ref() {
                            self.state = ConnectionState::Closed;
                            return Poll::Ready(Ok(()));
                        }
                    }

                    // Client-side close. We respond with the same close frame.
                    return self.close(frame);
                }
                // Cannot receive a raw frame.
                Message::Frame(_) => unreachable!(),
            }

            return Poll::Ready(Ok(()));
        }

        if let Some(timeout) = &mut this.pong_timeout {
            let timeout = unsafe { Pin::new_unchecked(timeout) };

            if timeout.poll(cx).is_ready() {
                log::debug!("Connection timed out");
                return self.close(None);
            }
        }

        if self.as_mut().poll_tick(cx).is_ready() {
            return Poll::Ready(Ok(()));
        }

        let this = self.as_mut().project();
        if let Poll::Ready(res) = this.changed.poll_next(cx) {
            let buf = match res {
                Some(Ok(event)) => Response::from(event).to_bytes(),
                // Lagged
                Some(Err(_)) => Response::Error(ErrorResponse::Lagged).to_bytes(),
                None => unreachable!(),
            };

            self.init_write(Message::Binary(buf));
            return self.poll_write(cx);
        }

        Poll::Pending
    }

    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        log::trace!("Connection.poll_write");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Write(_)));

        let mut this = self.as_mut().project();

        match this.stream.as_mut().poll_ready(cx) {
            Poll::Ready(Ok(())) => (),
            Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
            Poll::Pending => return Poll::Pending,
        }

        let msg = match &this.state {
            ConnectionState::Write(msg) => msg.clone(),
            _ => unreachable!(),
        };

        if let Err(err) = this.stream.start_send(msg) {
            log::debug!("Failed to write to sink: {}", err);
            return self.close(None);
        }

        self.init_read();
        Poll::Ready(Ok(()))
    }

    fn poll_tick(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        log::trace!("Connection.poll_tick");

        #[cfg(debug_assertions)]
        assert!(matches!(self.state, ConnectionState::Read));

        match self.ping_interval.poll_tick(cx) {
            Poll::Ready(_) => {
                // Peer must respond within 15s.
                self.pong_timeout = Some(time::sleep(Duration::from_secs(15)));

                self.init_write(Message::Ping(vec![0]));
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn init_read(&mut self) {
        let prev = self.state_str();

        self.state = ConnectionState::Read;

        log::trace!("Connection.state {} -> {}", prev, self.state_str());
    }

    fn init_write(&mut self, msg: Message) {
        let prev = self.state_str();

        self.state = ConnectionState::Write(msg);

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
            Request::UpdateMatch { index, nodes } => {
                if self.client_user.is_some() {
                    self.bracket.update(index, nodes);
                    None
                } else {
                    Some(Response::Error(ErrorResponse::Unauthorized))
                }
            }
            Request::ResetMatch { index } => {
                if self.client_user.is_some() {
                    self.bracket.reset(index as usize);
                    None
                } else {
                    Some(Response::Error(ErrorResponse::Unauthorized))
                }
            }
        }
    }

    fn state_str(&self) -> &'static str {
        match self.state {
            ConnectionState::Read => "Read",
            ConnectionState::Write(_) => "Write",
            ConnectionState::Close(_) => "Close",
            ConnectionState::Closed => "Closed",
        }
    }
}

/// Await state of the connection.
#[derive(Debug)]
enum ConnectionState {
    /// Waiting for incoming messages.
    Read,
    /// Waiting the sink to be ready.
    Write(Message),
    Close(Message),
    Closed,
}

impl<S> Future for Connection<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        log::trace!("Connection.poll");

        loop {
            let this = self.as_mut();

            match this.state {
                ConnectionState::Read => match this.poll_read(cx)? {
                    Poll::Ready(()) => continue,
                    Poll::Pending => return Poll::Pending,
                },
                ConnectionState::Write(_) => match this.poll_write(cx)? {
                    Poll::Ready(()) => continue,
                    Poll::Pending => return Poll::Pending,
                },
                ConnectionState::Close(_) => match this.poll_close(cx)? {
                    Poll::Ready(()) => continue,
                    Poll::Pending => return Poll::Pending,
                },
                ConnectionState::Closed => return Poll::Ready(Ok(())),
            }
        }
    }
}

unsafe impl<S> Send for Connection<S> where S: AsyncRead + AsyncWrite + Unpin + Send {}

// Both `TcpStream` and `UnixStream` are Sync, unlike Upgraded for some reason.
unsafe impl<S> Sync for Connection<S> where S: AsyncRead + AsyncWrite + Unpin {}
