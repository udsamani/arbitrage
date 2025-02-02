use std::collections::HashSet;

use common::{ArbitrageError, ArbitrageResult, Context, WorkerRef};
use models::{deribit::{DeribitChannelMessage, DeribitRequest, DeribitRequestMethod, DeribitRequestParams, DeribitResponse}, ProductSubscription};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

use super::get_products_to_subscibe;

pub struct DeribitExchangeAdapter {
    context: Context,
    ws_client: WsClient,
    products_to_subscribe: HashSet<ProductSubscription>
}

impl DeribitExchangeAdapter {
    /// Create a new DeribitExchangeAdapter
    ///
    /// The environment variables required are:
    /// - `DERIBIT_WS_URL`: The URL of the Deribit WebSocket API
    /// - `DERIBIT_PRODUCTS_TO_SUBSCRIBE`: The products to subscribe to, separated by commas
    /// - `DERIBIT_HEARTBEAT_MILLIS`: The heartbeat interval in milliseconds
    pub fn new(context: Context) -> ArbitrageResult<Self> {
        let products_to_subscribe = context.config.get_string("deribit_products_to_subscribe")?;
        let products_to_subscribe = get_products_to_subscibe(&products_to_subscribe);

        let ws_url = context.config.get_string("deribit_ws_url")?;
        let heartbeat_millis = context.config.get_int("deribit_heartbeat_millis")?;
        let ws_client = WsClient::new(ws_url, heartbeat_millis as u64);

        Ok(Self { context, ws_client, products_to_subscribe })
    }

    pub fn callback(&self) -> DeribitExchangeCallback {
        DeribitExchangeCallback {
            _context: self.context.clone(),
            ws_client: self.ws_client.clone(),
            products_to_subscribe: self.products_to_subscribe.clone(),
            inflight_subscription_requests: HashSet::new(),
        }
    }

    pub fn worker(&mut self, callback: DeribitExchangeCallback) -> WorkerRef {
        Box::new(
            self.ws_client.consumer(self.context.with_name("deribit-ws-consumer"), callback)
        )
    }

}


/// Implements the WsCallback for the Deribit exchange.
/// This is used to handle the incoming messages from the Deribit exchange.
#[derive(Clone)]
pub struct DeribitExchangeCallback {
    _context: Context,
    ws_client: WsClient,
    products_to_subscribe: HashSet<ProductSubscription>,
    inflight_subscription_requests: HashSet<String>
}


impl DeribitExchangeCallback {
    pub fn subscribe_products(&mut self) -> ArbitrageResult<()> {
        for (index, product) in self.products_to_subscribe.iter().enumerate() {
            if !product.subscribed && !self.inflight_subscription_requests.contains(&product.product_id) {
                self.inflight_subscription_requests.insert(product.product_id.clone());
                let request = DeribitRequest {
                    jsonrpc: "2.0".to_string(),
                    method: DeribitRequestMethod::PublicSubscribe,
                    params: DeribitRequestParams::Channels(vec![product.product_id.clone()]),
                    id: format!("{}", index)
                };
                // TODO: handle errors better
                let json = serde_json::to_string(&request).unwrap();
                self.ws_client.write(Message::Text(Utf8Bytes::from(&json)))?;
            }
        }
        Ok(())
    }
}


#[async_trait::async_trait]
impl WsCallback for DeribitExchangeCallback {

    async fn on_message(&mut self, message: Message, _received_time: jiff::Timestamp) -> ArbitrageResult<()> {


        match message {
            Message::Text(text) => {
                let result = serde_json::from_str::<DeribitResponse>(&text);
                match result {
                    Ok(response) => {
                        log::info!("received deribit response: {:?}", response);
                    }
                    Err(_) => {
                        let result = serde_json::from_str::<DeribitChannelMessage>(&text);
                        match result {
                            Ok(channel_message) => {
                                log::info!("received deribit channel message: {:?}", channel_message);
                            }
                            Err(e) => {
                                log::error!("error parsing deribit channel message: {}", e);
                            }
                        }
                    }
                }
            }
            Message::Close(close) => {
                if let Some(reason) = close {
                    log::error!("Deribit connection closed: {}", reason);
                } else {
                    log::error!("Deribit connection closed");
                }
            }
            _ => {
                return Err(ArbitrageError::Warning(format!("received unexpected message: {:?}", message)));
            }
        }

        Ok(())
    }

    async fn on_connect(&mut self, _timestamp: jiff::Timestamp) -> ArbitrageResult<()> {
        log::info!("connected to deribit");
        self.subscribe_products()?;
        Ok(())
    }

    fn on_disconnect(&mut self) -> ArbitrageResult<()> {
        log::info!("disconnected from deribit");
        Ok(())
    }

    fn on_heartbeat(&mut self) -> ArbitrageResult<()> {
        log::debug!("heartbeat from deribit");
        Ok(())
    }
}
