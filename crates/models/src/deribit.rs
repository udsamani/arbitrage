use std::fmt;

use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{de::{Error, SeqAccess, Visitor}, Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeribitRequest {
    pub method: DeribitRequestMethod,
    pub id: String,
    pub jsonrpc: String,
    pub params: DeribitRequestParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeribitResponse {
    pub jsonrpc: String,
    pub id: String,
    pub result: Vec<String>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeribitRequestMethod {
    #[serde(rename = "public/subscribe")]
    PublicSubscribe,
    #[serde(rename = "public/unsubscribe")]
    PublicUnsubscribe,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeribitRequestParams {
    Channels(Vec<String>),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeribitChannelMessage {
    pub jsonrpc: String,
    pub method: DeribitResponseMethod,
    pub params: DerbitResponseParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerbitResponseParams {
    pub channel: String,
    pub data: DeribitChannelData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DeribitResponseMethod {
    Subscription,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeribitChannelData {
    OrderBook(DeribitOrderBook),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeribitOrderBook {
    pub instrument_name: String,
    pub timestamp: u64,
    pub asks: Vec<OrderBookEntry>,
    pub bids: Vec<OrderBookEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrderBookEntry {
    pub price: Decimal,
    pub amount: Decimal,
}


impl<'de> Deserialize<'de> for OrderBookEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OrderBookTopVisitor;


        impl<'de> Visitor<'de> for OrderBookTopVisitor {
            type Value = OrderBookEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an array with two elements")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<OrderBookEntry, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let price = Decimal::from_f64(
                    seq.next_element::<f64>()?
                        .ok_or_else(|| Error::invalid_length(0, &self))?
                ).ok_or_else(|| Error::custom("Failed to convert f64 to Decimal"))?;

                let amount = Decimal::from_f64(
                    seq.next_element::<f64>()?
                        .ok_or_else(|| Error::invalid_length(1, &self))?
                ).ok_or_else(|| Error::custom("Failed to convert f64 to Decimal"))?;

                Ok(OrderBookEntry { price, amount })
            }
        }

        deserializer.deserialize_seq(OrderBookTopVisitor)
    }
}




#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_serialize_request() {
        let request = DeribitRequest {
            method: DeribitRequestMethod::PublicSubscribe,
            id: "1".to_string(),
            jsonrpc: "2.0".to_string(),
            params: DeribitRequestParams::Channels(vec!["book.BTC-10MAY24-66000-C.none.20.100ms".to_string()]),
        };

        let serialized = serde_json::to_value(&request).unwrap();

        let expected = serde_json::json!({
            "method": "public/subscribe",
            "id": "1",
            "jsonrpc": "2.0",
            "params": {
                "channels": ["book.BTC-10MAY24-66000-C.none.20.100ms"]
            }
        });

        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_serialize_response() {
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": "1",
            "result": ["book.BTC-10MAY24-66000-C.none.20.100ms"]
        });

        let deserialized = serde_json::from_value::<DeribitResponse>(response).unwrap();

        assert_eq!(deserialized.jsonrpc, "2.0");
        assert_eq!(deserialized.id, "1");
        assert_eq!(deserialized.result, vec!["book.BTC-10MAY24-66000-C.none.20.100ms"]);
    }

    #[test  ]
    fn test_deserialize_order_book() {

        let json = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "subscription",
            "params": {
                "channel": "book.BTC-10MAY24-66000-C.none.20.100ms",
                "data": {
                    "instrument_name": "BTC-10MAY24-66000-C",
                    "timestamp": 1717219200,
                    "asks": [
                        [66000, 100]
                    ],
                    "bids": [
                        [65000, 100]
                    ],
                },
            }
        });

        let deserialized: DeribitChannelMessage = serde_json::from_value(json).unwrap();

        assert_eq!(deserialized.params.channel, "book.BTC-10MAY24-66000-C.none.20.100ms");
        assert_eq!(deserialized.method, DeribitResponseMethod::Subscription);

        let DeribitChannelData::OrderBook(order_book) = deserialized.params.data;
        let asks = order_book.asks;
        let bids = order_book.bids;

        assert_eq!(asks.len(), 1);
        assert_eq!(bids.len(), 1);

        assert_eq!(asks[0].price, Decimal::from(66000));
        assert_eq!(asks[0].amount, Decimal::from(100));

        assert_eq!(bids[0].price, Decimal::from(65000));
        assert_eq!(bids[0].amount, Decimal::from(100));
    }

}
