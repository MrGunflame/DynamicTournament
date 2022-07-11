use crate::Error;

use std::borrow::Cow;

#[cfg(target_family = "wasm")]
use futures::SinkExt;

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
    pub async fn send<T>(&mut self, msg: T)
    where
        T: Into<WebSocketMessage>,
    {
        let msg = msg.into();
        log::debug!("Sending {:?}", msg);

        #[cfg(target_family = "wasm")]
        let _ = self.inner.send(msg).await;
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

        Ok(WebSocket::new(&self.uri, handler)?)
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

    use futures::channel::mpsc;
    use futures::{select, SinkExt, StreamExt};
    use reqwasm::websocket::Message;
    use wasm_bindgen_futures::spawn_local;

    #[derive(Clone, Debug)]
    pub struct WebSocket {
        tx: mpsc::Sender<WebSocketMessage>,
    }

    impl WebSocket {
        pub fn new(uri: &str, mut handler: Box<dyn EventHandler>) -> Result<Self, Error> {
            let ws = reqwasm::websocket::futures::WebSocket::open(uri)?;

            let (tx, mut rx) = mpsc::channel(32);

            spawn_local(async move {
                let mut ws = ws.fuse();

                loop {
                    select! {
                        // Writer
                        msg = rx.next() => {
                            match msg {
                                Some(WebSocketMessage::Bytes(buf)) => {
                                    log::debug!("Sending bytes to ws peer: {:?}", buf);

                                    match ws.send(Message::Bytes(buf)).await {
                                        Ok(_) => (),
                                        Err(err) => {
                                            log::debug!("Failed to send buffer: {:?}", err);
                                            break;
                                        }
                                    }
                                }
                                Some(WebSocketMessage::Text(string)) => {
                                    match ws.send(Message::Text(string)).await {
                                        Ok(_) => (),
                                        Err(err) => {
                                            log::debug!("Failed to send buffer: {:?}", err);
                                            break;
                                        }
                                    }
                                }
                                Some(WebSocketMessage::Close) => {
                                    break;
                                }
                                None => {
                                    log::debug!("Writer done");

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

                                    handler.dispatch(WebSocketMessage::Close);
                                    break;
                                }
                            }
                        }
                    }
                }

                let _ = ws.into_inner().close(None, None);
                log::debug!("Dropped ws");
            });

            Ok(Self { tx })
        }

        pub async fn send(&mut self, msg: WebSocketMessage) {
            let _ = self.tx.send(msg).await;
        }
    }
}
