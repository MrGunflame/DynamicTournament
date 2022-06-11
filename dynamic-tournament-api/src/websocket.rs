use std::marker::PhantomData;

use bincode::Options;
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};

#[cfg(target_family = "wasm")]
use gloo_utils::errors::JsError;
#[cfg(target_family = "wasm")]
use reqwasm::websocket::Message;
#[cfg(target_family = "wasm")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Debug)]
pub struct WebSocket<T>
where
    T: Serialize + DeserializeOwned + 'static,
{
    tx: mpsc::Sender<Vec<u8>>,
    _marker: PhantomData<mpsc::Sender<T>>,
}

impl<T> WebSocket<T>
where
    T: Serialize + DeserializeOwned + 'static,
{
    #[cfg(target_family = "wasm")]
    pub fn new(uri: &str, mut handler: Box<dyn EventHandler<T>>) -> Result<Self, JsError> {
        log::debug!("Connecting to {}", uri);

        let ws = match reqwasm::websocket::futures::WebSocket::open(uri) {
            Ok(ws) => ws,
            Err(err) => return Err(err),
        };

        let (mut writer, mut reader) = ws.split();

        let (tx, mut rx) = mpsc::channel(32);

        log::debug!("Connected to {}", uri);

        // Writer task
        spawn_local(async move {
            while let Some(bytes) = rx.next().await {
                log::debug!("Writing websocket frame: {:?}", bytes);

                writer.send(Message::Bytes(bytes)).await.unwrap();
            }

            let _ = writer.close().await;
            log::debug!("Dropped ws writer");
        });

        spawn_local(async move {
            while let Some(msg) = reader.next().await {
                match msg {
                    Ok(Message::Bytes(bytes)) => {
                        log::debug!("Received message from ws peer: {:?}", bytes);

                        let msg = bincode::DefaultOptions::new()
                            .with_little_endian()
                            .with_varint_encoding()
                            .deserialize(&bytes)
                            .unwrap();

                        handler.dispatch(msg);
                    }
                    Ok(Message::Text(_)) => {}
                    Err(err) => {
                        log::error!("{:?}", err);
                    }
                }
            }

            log::debug!("Dropped ws reader");
        });

        Ok(Self {
            tx,
            _marker: PhantomData,
        })
    }

    pub async fn send(&mut self, msg: &T) -> Result<(), bincode::Error> {
        let bytes = bincode::DefaultOptions::new()
            .with_little_endian()
            .with_varint_encoding()
            .serialize(msg)?;

        let _ = self.tx.send(bytes).await;
        Ok(())
    }

    pub async fn close(&mut self) {}
}

pub trait EventHandler<T>
where
    T: Serialize + DeserializeOwned + 'static,
{
    fn dispatch(&mut self, msg: T);
}

pub struct WebSocketBuilder<T> {
    uri: String,
    handler: Option<Box<dyn EventHandler<T>>>,
}

impl<T> WebSocketBuilder<T>
where
    T: Serialize + DeserializeOwned + 'static,
{
    pub fn new(uri: String) -> Self {
        Self { uri, handler: None }
    }

    pub fn handler(mut self, handler: Box<dyn EventHandler<T>>) -> Self {
        self.handler = Some(handler);
        self
    }

    #[cfg(target_family = "wasm")]
    pub fn build(self) -> Result<WebSocket<T>, crate::Error> {
        let handler = match self.handler {
            Some(handler) => handler,
            None => Box::new(DefaultHandler),
        };

        Ok(WebSocket::new(&self.uri, handler)?)
    }

    #[cfg(not(target_family = "wasm"))]
    pub fn build(self) -> Result<WebSocket<T>, crate::Error> {
        unimplemented!()
    }
}

struct DefaultHandler;

impl<T> EventHandler<T> for DefaultHandler
where
    T: Serialize + DeserializeOwned + 'static,
{
    fn dispatch(&mut self, _msg: T) {}
}
