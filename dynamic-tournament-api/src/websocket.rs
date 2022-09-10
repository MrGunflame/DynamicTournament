use crate::Error;

use std::borrow::Cow;
use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug)]
pub struct WebSocketError(Box<dyn std::error::Error>);

impl Display for WebSocketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// A WebSocket connection.
///
/// The connection is automatically closed when all `WebSocket` instances are dropped.
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct WebSocket {
    #[cfg(not(target_family = "wasm"))]
    #[allow(unused)]
    inner: (),
    #[cfg(target_family = "wasm")]
    inner: wasm::WebSocket,
}

impl WebSocket {
    /// Opens a new `WebSocket` connection using the given `uri`.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] if creating the connection fails.
    #[allow(unused)]
    #[allow(clippy::let_unit_value)]
    #[inline]
    pub fn new(uri: &str, handler: Box<dyn EventHandler>) -> Result<Self, Error> {
        log::debug!("Connecting to {}", uri);

        #[cfg(not(target_family = "wasm"))]
        let inner = ();

        #[cfg(target_family = "wasm")]
        let inner = wasm::WebSocket::new(uri, handler)?;

        Ok(Self { inner })
    }

    /// Writes a [`WebSocketMessage`] into the connection. Note that `send` flushes its content
    /// immediately.
    #[allow(unused)]
    #[inline]
    pub async fn send<T>(&mut self, msg: T) -> Result<(), WebSocketError>
    where
        T: Into<WebSocketMessage>,
    {
        let msg = msg.into();
        log::debug!("Sending {:?}", msg);

        #[cfg(not(target_family = "wasm"))]
        unimplemented!();

        #[cfg(target_family = "wasm")]
        match self.inner.send(msg).await {
            Ok(()) => Ok(()),
            Err(err) => Err(WebSocketError(Box::new(err))),
        }
    }
}

/// Receiver for messages from a [`WebSocket`].
pub trait EventHandler {
    fn dispatch(&mut self, msg: WebSocketMessage);
}

/// Builder for a [`WebSocket`].
pub struct WebSocketBuilder {
    uri: Cow<'static, str>,
    handler: Option<Box<dyn EventHandler>>,
}

impl WebSocketBuilder {
    /// Creates a new `WebSocketBuilder` using the given `uri` for the connection.
    pub fn new<T>(uri: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        Self {
            uri: uri.into(),
            handler: None,
        }
    }

    /// Sets the [`EventHandler`] for the `WebSocket`.
    pub fn handler(mut self, handler: Box<dyn EventHandler>) -> Self {
        self.handler = Some(handler);
        self
    }

    /// Consumes the `WebSocketBuilder` and opens a new [`WebSocket`] using the parameters
    /// provided by the builder.
    ///
    /// # Errors
    ///
    /// Returns an [`enum@Error`] when creating a new [`WebSocket `] fails. For more details see
    /// [`WebSocket::new`].
    pub fn build(self) -> Result<WebSocket, Error> {
        let handler = match self.handler {
            Some(handler) => handler,
            None => Box::new(DefaultHandler),
        };

        WebSocket::new(&self.uri, handler)
    }
}

struct DefaultHandler;

impl EventHandler for DefaultHandler {
    fn dispatch(&mut self, _msg: WebSocketMessage) {}
}

/// A message that can be sent or received from a [`WebSocket`].
#[derive(Clone, Debug)]
pub enum WebSocketMessage {
    Bytes(Vec<u8>),
    Text(String),
    Close,
}

impl From<Vec<u8>> for WebSocketMessage {
    #[inline]
    fn from(buf: Vec<u8>) -> Self {
        Self::Bytes(buf)
    }
}

impl From<String> for WebSocketMessage {
    #[inline]
    fn from(string: String) -> Self {
        Self::Text(string)
    }
}

#[cfg(target_family = "wasm")]
mod wasm {
    use super::{EventHandler, WebSocketMessage};
    use crate::Error;

    use std::fmt::{self, Display, Formatter};

    use futures::channel::{mpsc, oneshot};
    use futures::{select, SinkExt, StreamExt};
    use reqwasm::websocket::Message;
    use wasm_bindgen_futures::spawn_local;

    #[derive(Clone, Debug)]
    pub struct WebSocket {
        tx: mpsc::Sender<(
            WebSocketMessage,
            oneshot::Sender<Result<(), WebSocketError>>,
        )>,
    }

    impl WebSocket {
        pub fn new(uri: &str, mut handler: Box<dyn EventHandler>) -> Result<Self, Error> {
            let ws = reqwasm::websocket::futures::WebSocket::open(uri)?;

            let (tx, mut rx) = mpsc::channel::<(
                WebSocketMessage,
                oneshot::Sender<Result<(), WebSocketError>>,
            )>(32);

            spawn_local(async move {
                let mut ws = ws.fuse();

                loop {
                    select! {
                        // Writer
                        msg = rx.next() => {
                            match msg {
                                Some((WebSocketMessage::Bytes(buf), tx)) => {
                                    log::debug!("Sending bytes to ws peer: {:?}", buf);

                                    match ws.send(Message::Bytes(buf)).await {
                                        Ok(_) => {
                                            let _ = tx.send(Ok(()));
                                        },
                                        Err(err) => {
                                            log::debug!("Failed to send buffer: {:?}", err);
                                            let _ = tx.send(Err(err.into()));
                                            break;
                                        }
                                    }
                                }
                                Some((WebSocketMessage::Text(string), tx)) => {
                                    match ws.send(Message::Text(string)).await {
                                        Ok(_) => {
                                            let _ = tx.send(Ok(()));
                                        },
                                        Err(err) => {
                                            log::debug!("Failed to send buffer: {:?}", err);
                                            let _ = tx.send(Err(err.into()));
                                            break;
                                        }
                                    }
                                }
                                Some((WebSocketMessage::Close, tx)) => {
                                    let _ = tx.send(Ok(()));
                                    break;
                                }
                                None => {
                                    break;
                                }
                            }
                        }

                        // Reader
                        msg = ws.next() => {
                            match msg {
                                Some(Ok(Message::Bytes(buf))) => {
                                    log::debug!("Received bytes from ws peer: {:?}", buf);
                                    handler.dispatch(WebSocketMessage::Bytes(buf));
                                }
                                Some(Ok(Message::Text(string))) => {
                                    log::debug!("Received text from ws peer: {:?}", string);
                                    handler.dispatch(WebSocketMessage::Text(string));
                                }
                                Some(Err(err)) => {
                                    log::error!("Failed to read from ws: {:?}", err);
                                    break;
                                }
                                None => {
                                    log::debug!("ws reader closed");
                                    break;
                                }
                            }
                        }
                    }
                }

                let _ = ws.into_inner().close(None, None);
                handler.dispatch(WebSocketMessage::Close);
                log::debug!("Dropped ws");
            });

            Ok(Self { tx })
        }

        pub async fn send(&mut self, msg: WebSocketMessage) -> Result<(), WebSocketError> {
            let (tx, rx) = oneshot::channel();
            let _ = self.tx.send((msg, tx)).await;
            match rx.await {
                Ok(res) => res,
                Err(_) => Err(WebSocketError::Closed),
            }
        }
    }

    #[derive(Debug)]
    pub enum WebSocketError {
        WebSocket(reqwasm::websocket::WebSocketError),
        Closed,
    }

    impl Display for WebSocketError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            match self {
                Self::WebSocket(ws) => ws.fmt(f),
                Self::Closed => f.write_str("closed"),
            }
        }
    }

    impl std::error::Error for WebSocketError {}

    impl From<reqwasm::websocket::WebSocketError> for WebSocketError {
        fn from(src: reqwasm::websocket::WebSocketError) -> Self {
            Self::WebSocket(src)
        }
    }
}
