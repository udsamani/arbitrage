use common::ArbitrageResult;
use models::InternalMessage;
use tokio::sync::broadcast::Receiver;
use futures_util::{stream::StreamExt, SinkExt};

pub struct WebSocket {
    receiver: Receiver<InternalMessage>,
}


impl WebSocket {
    pub fn new(receiver: Receiver<InternalMessage>) -> Self {
        Self { receiver }
    }

    pub async fn serve(&mut self, ws: warp::ws::WebSocket) -> ArbitrageResult<()> {
        // For simplicity, we are not parsing any request or looking for any subscription.
        // If a connection is established, we will send arbitrage opportunities to the client.
        let (mut ws_tx, mut ws_rx) = ws.split();
        log::info!("a new websocket connection established");
        loop {
            tokio::select! {
                _ = ws_rx.next() => {
                    if let Some(Ok(msg)) = ws_rx.next().await {
                        if msg.is_close() {
                            log::info!("websocket connection closed as received close message");
                            return Ok(());
                        }

                    }
                }
                message = self.receiver.recv() => {
                    match message {
                        Ok(msg) => {
                            match msg {
                                InternalMessage::ArbitrageOpportunity(opportunity) => {
                                    match serde_json::to_string(&opportunity) {
                                        Ok(json) => {
                                            ws_tx.send(warp::ws::Message::text(json)).await.unwrap();
                                        }
                                        Err(e) => {
                                            log::error!("error serializing arbitrage opportunity: {}", e);
                                        }
                                    }
                                }
                                _ => {
                                    log::error!("received unknown message from broadcaster, only arbitrage opportunities are supported");
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("error receiving message from broadcaster: {}", e);
                            break;
                        }
                    }
                }
            }
        }
        Ok(())

    }
}
