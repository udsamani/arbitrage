use std::fmt;

use rust_decimal::Decimal;
use serde::{de::{SeqAccess, Visitor, Error}, Deserialize, Deserializer, Serialize};


#[derive(Clone, Serialize, Deserialize)]
pub struct OkexRequest {
    pub op: OkexOperation,
    pub args: Vec<OkexArg>,
}


#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OkexOperation {
    Subscribe,
    Unsubscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OkexEvent {
    Subscribe,
    Unsubscribe,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub instance_id: String,
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexResponse {
    pub event: OkexEvent,
    #[serde(rename = "connId")]
    pub connection_id: String,
    #[serde(flatten)]
    pub response_data: OkexResponseData,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OkexResponseData {
    Subscribe(OkexSubscribeResponse),
    Error(OkexError),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexSubscribeResponse {
    pub arg: OkexArg,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexError {
    pub code: String,
    #[serde(rename = "msg")]
    pub message: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OkexAction {
    Snapshot,
    Update,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexMessage {
    pub action: OkexAction,
    pub arg: OkexArg,
    pub data: Vec<OkexData>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OkexData {
    #[serde(rename = "ts", deserialize_with = "deserialize_timestamp")]
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
                formatter.write_str("an array with four elements")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<OrderBookEntry, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let price: Decimal = seq
                    .next_element::<String>()?
                    .ok_or_else(|| Error::invalid_length(0, &self))?
                    .parse()
                    .map_err(Error::custom)?;

                let amount: Decimal = seq
                    .next_element::<String>()?
                    .ok_or_else(|| Error::invalid_length(1, &self))?
                    .parse()
                    .map_err(Error::custom)?;


                // Ignore any additional elements
                while (seq.next_element::<String>()?).is_some() {}

                Ok(OrderBookEntry {
                    price,
                    amount,
                })
            }
        }

        deserializer.deserialize_seq(OrderBookTopVisitor)
    }
}


fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let timestamp: String = String::deserialize(deserializer)?;
    timestamp.parse::<u64>().map_err(Error::custom)
}




#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_serialize_subscribe() {
        let request = OkexRequest {
            op: OkexOperation::Subscribe,
            args: vec![OkexArg { channel: "books".to_string(), instance_id: "BTC-USDT".to_string() }],
        };


        let expected_json = serde_json::json!({
            "op": "subscribe",
            "args": [
                {
                    "channel": "books",
                    "instId": "BTC-USDT"
                }
            ]
        });

        let actual_json = serde_json::to_value(&request).unwrap();
        assert_eq!(expected_json, actual_json);
    }

    #[test]
    fn test_deserialize_subscribe_response() {
        let response_json = serde_json::json!({
            "event": "subscribe",
            "arg": {
                "channel": "books",
                "instId": "BTC-USDT"
            },
            "connId": "1234567890"
        });

        let response = serde_json::from_value::<OkexResponse>(response_json).unwrap();
        assert_eq!(response.event, OkexEvent::Subscribe);
        match response.response_data {
            OkexResponseData::Subscribe(subscribe_response) => {
                assert_eq!(subscribe_response.arg.channel, "books");
                assert_eq!(subscribe_response.arg.instance_id, "BTC-USDT");
            }
            _ => panic!("Expected Subscribe response"),
        }
        assert_eq!(response.connection_id, "1234567890");
    }

    #[test]
    fn test_deserialize_error() {
        let response_json = serde_json::json!({
            "event": "error",
            "code": "1234567890",
            "msg": "Error message",
            "connId": "1234567890"
        });

        let response = serde_json::from_value::<OkexResponse>(response_json).unwrap();
        assert_eq!(response.event, OkexEvent::Error);
        match response.response_data {
            OkexResponseData::Error(error) => {
                assert_eq!(error.code, "1234567890");
                assert_eq!(error.message, "Error message");
            }
            _ => panic!("Expected Error response"),
        }
        assert_eq!(response.connection_id, "1234567890");
    }

    #[test]
    fn test_deserialize_snapshot_message() {
        let message_json = serde_json::json!({
            "action": "snapshot",
            "arg": {
                "channel": "books",
                "instId": "BTC-USDT"
            },
            "data": [
                {
                    "ts": "1234567890",
                    "checksum": 1234567890,
                    "prevSeqId": null,
                    "seqId": 1234567890,
                    "asks": [["84000.00000000", "1.00000000", "0", "1.0"], ["83000.00000000", "1.00000000", "0", "1.0"]],
                    "bids": [["84000.00000000", "1.00000000", "0", "1.0"], ["83000.00000000", "1.00000000", "0", "1.0"]]
                }
            ]
        });

        let message = serde_json::from_value::<OkexMessage>(message_json).unwrap();
        assert_eq!(message.action, OkexAction::Snapshot);
        assert_eq!(message.arg.channel, "books");
        assert_eq!(message.arg.instance_id, "BTC-USDT");
        assert_eq!(message.data.len(), 1);
        let data = message.data[0].clone();
        assert_eq!(data.timestamp, 1234567890);
        assert_eq!(data.asks.len(), 2);
        assert_eq!(data.bids.len(), 2);
        assert_eq!(data.asks[0].price, Decimal::from(84000));
        assert_eq!(data.asks[0].amount, Decimal::from(1));
        assert_eq!(data.asks[1].price, Decimal::from(83000));
        assert_eq!(data.asks[1].amount, Decimal::from(1));
        assert_eq!(data.bids[0].price, Decimal::from(84000));
        assert_eq!(data.bids[0].amount, Decimal::from(1));
        assert_eq!(data.bids[1].price, Decimal::from(83000));
    }

    #[test]
    fn test_deserialize_update_message() {
        let message_json = serde_json::json!({
            "action": "update",
            "arg": {
                "channel": "books",
                "instId": "BTC-USDT"
            },
            "data": [
                {
                    "ts": "1234567890",
                    "asks": [["84000.00000000", "1.00000000"], ["83000.00000000", "1.00000000"]],
                    "bids": [["84000.00000000", "1.00000000"], ["83000.00000000", "1.00000000"]]
                }
            ]
        });

        let message = serde_json::from_value::<OkexMessage>(message_json).unwrap();
        assert_eq!(message.action, OkexAction::Update);
        assert_eq!(message.arg.channel, "books");
        assert_eq!(message.arg.instance_id, "BTC-USDT");
        assert_eq!(message.data.len(), 1);
        let data = message.data[0].clone();
        assert_eq!(data.timestamp, 1234567890);
        assert_eq!(data.asks.len(), 2);
        assert_eq!(data.bids.len(), 2);
        assert_eq!(data.asks[0].price, Decimal::from(84000));
        assert_eq!(data.asks[0].amount, Decimal::from(1));
        assert_eq!(data.asks[1].price, Decimal::from(83000));
        assert_eq!(data.asks[1].amount, Decimal::from(1));
        assert_eq!(data.bids[0].price, Decimal::from(84000));
        assert_eq!(data.bids[0].amount, Decimal::from(1));
        assert_eq!(data.bids[1].price, Decimal::from(83000));
        assert_eq!(data.bids[1].amount, Decimal::from(1));
    }
}
