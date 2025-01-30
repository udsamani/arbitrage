use std::time::Duration;
use futures_util::StreamExt;

use jiff::Timestamp;
use tokio::io;

use common::{ArbitrageError, ArbitrageResult, Backoff, Context};
use tokio_tungstenite::WebSocketStream;

use crate::WsCallback;

pub struct WsConsumer<C>
where
    C: WsCallback,
{
    pub client_id: String,
    pub ws_url: String,
    pub callback: C,
    pub heartbeat_millis: u64,
    pub backoff: Backoff,
    pub context: Context,
}

#[allow(unused)]
impl<C> WsConsumer<C>
where
    C: WsCallback,
{
    pub async fn run(&mut self) -> ArbitrageResult<String> {
        let context = self.context.clone();
        loop {
            match self.backoff.next() {
                Some(delay_secs) => {
                    if delay_secs > 0 {
                        tokio::time::sleep(Duration::from_secs(delay_secs as u64)).await;
                    }
                }
                None => {
                    return Err(ArbitrageError::GenericError(format!(
                        "failed to connect to websocket after {} attempts",
                        self.backoff.get_iteration_count()
                    )));
                }
            }

            log::info!("connecting to websocket: {}", self.ws_url);
            let ws_stream = match tokio_tungstenite::connect_async(&self.ws_url).await {
                Ok((ws_stream, _)) => {
                    log::info!("connected to websocket: {}", &self.ws_url);
                    self.backoff.reset();
                    ws_stream
                }
                Err(e) => {
                    log::warn!(
                        "failed to connect to websocket: {} with error: {}",
                        &self.ws_url,
                        e
                    );
                    continue;
                }
            };

            let stream_result = self.stream(ws_stream).await;
            self.on_disconnect()?;

            match stream_result {
                Ok(_) => {
                    log::warn!("websocket {} disconnected", self.client_id);
                }
                Err(ArbitrageError::Exit) => {

                }
                Err(ArbitrageError::UnrecoverableError(e)) => {
                    log::error!("unrecoverable error: {}", e);
                    return Err(ArbitrageError::UnrecoverableError(e));
                }
                Err(ArbitrageError::Warning(e)) => {
                    log::error!("websocket {} scheduled reconnect", self.client_id);
                }
                Err(e) => {
                    log::error!("error while streaming websocket: {}", e);
                }
            }
        }
    }

    async fn stream<S>(&mut self, mut ws_stream: WebSocketStream<S>) -> ArbitrageResult<()>
    where
        S: io::AsyncRead + io::AsyncWrite + Unpin + Send + 'static,
    {
        self.on_connect().await?;
        let mut app = self.context.app.subscribe();
        let mut num_messages_since_last_heartbeat = 0;
        let mut heartbeat = tokio::time::interval(Duration::from_millis(self.heartbeat_millis));

        loop {
            tokio::select! {
                _ = app.recv() => {
                    return Err(ArbitrageError::Exit);
                }
                result = ws_stream.next() => {
                    match result {
                        Some(result) => {
                            let received_time = Timestamp::now();
                            num_messages_since_last_heartbeat += 1;
                            match result {
                                Ok(message) => {
                                    self.callback.on_message(message, received_time).await?;
                                }
                                Err(e) => {
                                    return Err(ArbitrageError::GenericError(format!("error while streaming websocket: {}", e)));
                                }
                            };
                        }
                        None => {
                            return Err(ArbitrageError::GenericError("websocket stream closed".to_string()));
                        }
                    }
                }
                _ = heartbeat.tick() => {
                    self.callback.on_heartbeat();
                    log::info!("{} received {} messages since last heartbeat", self.client_id, num_messages_since_last_heartbeat);
                    num_messages_since_last_heartbeat = 0;
                }
            }

        }
    }

    async fn on_connect(&mut self) -> ArbitrageResult<()> {
        let timestamp = Timestamp::now();
        self.callback.on_connect(timestamp).await
    }

    fn on_disconnect(&mut self) -> ArbitrageResult<()> {
        self.callback.on_disconnect()
    }
}
