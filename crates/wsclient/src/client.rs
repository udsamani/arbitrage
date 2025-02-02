use common::{ArbitrageError, ArbitrageResult, Backoff, Context, MpSc, SharedRef};
use tokio_tungstenite::tungstenite::Message;

use crate::{WsCallback, WsConsumer};

#[allow(unused)]
pub struct WsClient {
    ws_url: String,
    connected: SharedRef<bool>,
    mpsc: MpSc<Message>,
    client_id: String,
    heartbeat_millis: u64,
}


impl Clone for WsClient {
    fn clone(&self) -> Self {
        Self {
            ws_url: self.ws_url.clone(),
            connected: self.connected.clone(),
            mpsc: self.mpsc.clone(),
            client_id: self.client_id.clone(),
            heartbeat_millis: self.heartbeat_millis,
        }
    }
}

impl WsClient {
    pub fn new(ws_url: String, heartbeat_millis: u64) -> Self {
        let mpsc = MpSc::new(100);
        Self {
            ws_url,
           connected: SharedRef::new(false),
            client_id: "".to_string(),
            heartbeat_millis,
            mpsc,
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
        match self.mpsc.sender.try_send(message) {
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
        WsConsumer {
            client_id: self.client_id.clone(),
            ws_url: self.ws_url.clone(),
            callback,
            heartbeat_millis: self.heartbeat_millis,
            backoff: Backoff::default(),
            context,
            mpsc: self.mpsc.clone(),
        }
    }
}
