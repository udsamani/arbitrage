use common::{ArbitrageError, ArbitrageResult, Backoff, Context, SharedRef};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_tungstenite::tungstenite::Message;

use crate::{WsCallback, WsConsumer};

#[allow(unused)]
pub struct WsClient {
    ws_url: String,
    connected: SharedRef<bool>,
    sender: Sender<Message>,
    client_id: String,
    heartbeat_millis: u64,
    _receiver: Option<Receiver<Message>>,
}


impl Clone for WsClient {
    fn clone(&self) -> Self {
        Self {
            ws_url: self.ws_url.clone(),
            connected: self.connected.clone(),
            sender: self.sender.clone(),
            client_id: self.client_id.clone(),
            heartbeat_millis: self.heartbeat_millis,
            _receiver: None,
        }
    }
}

impl WsClient {
    pub fn new(ws_url: String, heartbeat_millis: u64) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);
        Self {
            ws_url,
            connected: SharedRef::new(false),
            sender,
            client_id: "".to_string(),
            heartbeat_millis,
            _receiver: Some(receiver),
        }
    }

    pub fn with_client_id(mut self, id: String) -> Self {
        self.client_id = id;
        self
    }

    pub fn ws_url(&self) -> &str {
        &self.ws_url
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.lock()
    }

    pub fn write(&self, message: Message) -> ArbitrageResult<()> {
        match self.sender.try_send(message) {
            Ok(_) => Ok(()),
            Err(e) => {
                Err(ArbitrageError::GenericError(format!("failed to send message to ws client: {}", e)))
            }
        }
    }

    pub fn close(&self) -> ArbitrageResult<()> {
        self.write(Message::Close(None))
    }

    pub fn consumer<C>(&mut self, context: Context, callback: C) -> WsConsumer<C>
    where
        C: WsCallback,
    {
        let receiver = self._receiver.take().unwrap();
        WsConsumer {
            client_id: self.client_id.clone(),
            ws_url: self.ws_url.clone(),
            callback,
            heartbeat_millis: self.heartbeat_millis,
            backoff: Backoff::default(),
            context,
            receiver: Some(receiver),
        }
    }
}
