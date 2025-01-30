use common::SharedRef;

#[allow(unused)]
#[derive(Clone)]
pub struct WsClient {
    ws_url: String,
    connected: SharedRef<bool>,
    client_id: String,
    heartbeat_millis: u64,
}

impl WsClient {
    pub fn new(ws_url: String, heartbeat_millis: u64) -> Self {
        Self {
            ws_url,
            connected: SharedRef::new(false),
            client_id: "".to_string(),
            heartbeat_millis,
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
}
