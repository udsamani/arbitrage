use common::ArbitrageResult;
use tokio_tungstenite::tungstenite::Message;

#[async_trait::async_trait]
pub trait WsCallback {
    async fn on_connect(&self, timestamp: jiff::Timestamp) -> ArbitrageResult<()>;
    async fn on_message(&self, message: Message, received_time: jiff::Timestamp) -> ArbitrageResult<()>;
    fn on_disconnect(&self) -> ArbitrageResult<()>;
    fn on_heartbeat(&self) -> ArbitrageResult<()>;
}
