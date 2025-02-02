use std::collections::HashSet;

use common::{ArbitrageError, ArbitrageResult, Context, WorkerRef};
use models::{okex::{OkexArg, OkexMessage, OkexOperation, OkexRequest, OkexResponse}, ProductSubscription};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use wsclient::{WsCallback, WsClient};

pub struct OkexExchangeAdapter {
    context: Context,
    ws_client: WsClient,
    products_to_subscribe: HashSet<ProductSubscription>,
}


pub fn get_products_to_subscibe(products_to_subscribe: &str) -> HashSet<ProductSubscription> {
    let mut products = HashSet::new();
    for product in products_to_subscribe.split(',') {
        products.insert(ProductSubscription {
            product_id: product.to_string(),
            subscribed: false,
        });
    }
    products
}

impl OkexExchangeAdapter {
    /// Create a new OkexExchangeAdapter
    ///
    /// The environment variables required are:
    /// - `OKEX_WS_URL`: The URL of the Okex WebSocket API
    /// - `OKEX_PRODUCTS_TO_SUBSCRIBE`: The products to subscribe to, separated by commas
    /// - `OKEX_HEARTBEAT_MILLIS`: The heartbeat interval in milliseconds
    pub fn new(context: Context) -> ArbitrageResult<Self> {
        let products_to_subscribe = context.config.get_string("okex_products_to_subscribe")
            .unwrap_or_default();
        let products_to_subscribe = get_products_to_subscibe(&products_to_subscribe);

        let ws_url = context.config.get_string("okex_ws_url")?;
        let heartbeat_millis = context.config.get_int("okex_heartbeat_millis")?;
        let ws_client = WsClient::new(ws_url, heartbeat_millis as u64);

        Ok(Self { context, ws_client, products_to_subscribe })
    }

    pub fn callback(&self) -> OkexExchangeCallback {
        OkexExchangeCallback {
            _context: self.context.clone(),
            ws_client: self.ws_client.clone(),
            products_to_subscribe: self.products_to_subscribe.clone(),
            inflight_subscription_requests: HashSet::new(),
        }
    }

    pub fn worker(&mut self, callback: OkexExchangeCallback) -> WorkerRef {
        Box::new(
            self.ws_client.consumer(self.context.with_name("okex-ws-consumer"), callback)
        )
    }
}


/// Implements the WsCallback for the Okex exchange.
/// This is used to handle the incoming messages from the Okex exchange.
#[derive(Clone)]
pub struct OkexExchangeCallback {
    _context: Context,
    ws_client: WsClient,
    products_to_subscribe: HashSet<ProductSubscription>,
    inflight_subscription_requests: HashSet<String>,
}


impl OkexExchangeCallback {
    pub fn subscribe_products(&mut self) -> ArbitrageResult<()> {
        let mut args = vec![];
        for product in self.products_to_subscribe.iter() {
            if !product.subscribed && !self.inflight_subscription_requests.contains(&product.product_id) {
                self.inflight_subscription_requests.insert(product.product_id.clone());
                args.push(OkexArg{
                    channel: "books".to_string(),
                    instance_id: product.product_id.clone(),
                });
            }
        }
        let message = OkexRequest{
            op: OkexOperation::Subscribe,
            args
        };

        //TODO: handle errors better
        let json = serde_json::to_string(&message).unwrap();
        self.ws_client.write(Message::Text(Utf8Bytes::from(&json)))?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl WsCallback for OkexExchangeCallback {
    async fn on_message(&mut self, message: Message, _received_time: jiff::Timestamp) -> ArbitrageResult<()> {

        match message {
            Message::Text(text) => {
                let result = serde_json::from_str::<OkexResponse>(&text);
                match result {
                    Ok(response) => {
                        log::info!("received okex response: {:?}", response);
                    }
                    Err(_) => {
                        // Parse it as channel message
                        let result = serde_json::from_str::<OkexMessage>(&text);
                        match result {
                            Ok(channel_message) => {
                                log::info!("received okex channel message: {:?}", channel_message);
                            }
                            Err(e) => {
                                log::error!("error parsing either okex response or channel message: {}", e);
                            }
                        }
                    }
                }
            }
            Message::Ping(ping) => {
                log::info!("received okex ping: {:?}", ping);
            }
            Message::Close(close) => {
                if let Some(reason) = close {
                    log::error!("Okex connection closed: {}", reason);
                } else {
                    log::error!("Okex connection closed");
                }
            }
            _ => {
                return Err(ArbitrageError::Warning(format!("received unexpected message: {:?}", message)));
            }
        }

        Ok(())
    }

    async fn on_connect(&mut self, _timestamp: jiff::Timestamp) -> ArbitrageResult<()> {
        log::info!("connected to Okex");
        self.subscribe_products()?;
        Ok(())
    }

    fn on_disconnect(&mut self) -> ArbitrageResult<()>  {
        log::info!("disconnected from Okex");
        Ok(())
    }

    fn on_heartbeat(&mut self) -> ArbitrageResult<()>  {
        log::debug!("heartbeat from Okex");
        Ok(())
    }
}
