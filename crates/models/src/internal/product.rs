use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Serialize;

use super::Exchange;



#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize)]
pub struct ExchangeProduct {
    pub product: Product,
    pub exchange: Exchange,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize)]
pub enum Product {
    Option {
        underlying: CryptoAsset,
        settlement: SettlementAsset,
        strike: Decimal,
        expiration: NaiveDate,
        option_type: OptionType,
    },
}


#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize)]
pub enum OptionType {
    Call,
    Put,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize)]
pub enum CryptoAsset {
    BTC,
    ETH
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize)]
pub enum SettlementAsset {
    USD,
}


impl Product {

    pub fn from_okex_exhchange(s: &str) -> Option<Self> {
        let parts = s.split('-').collect::<Vec<&str>>();
        if parts.len() != 5 {
            return None;
        }

        let underlying = match parts[0] {
            "BTC" => CryptoAsset::BTC,
            "ETH" => CryptoAsset::ETH,
            _ => return None,
        };

        let settlement = match parts[1] {
            "USD" => SettlementAsset::USD,
            _ => return None,
        };
        let strike = Decimal::from_str(parts[3]).unwrap_or_default();
        let expiration = NaiveDate::parse_from_str(parts[2], "%y%m%d").unwrap();

        let option_type = match parts[4] {
            "C" => OptionType::Call,
            "P" => OptionType::Put,
            _ => return None,
        };

        Some(Self::Option { underlying, settlement, strike, expiration, option_type })
    }

    pub fn from_deribit_exchange(s: &str) -> Option<Self> {
        let parts = s.split('-').collect::<Vec<&str>>();
        if parts.len() != 4 {
            return None;
        }

        let underlying = match parts[0] {
            "BTC" => CryptoAsset::BTC,
            "ETH" => CryptoAsset::ETH,
            _ => return None,
        };

        let settlement = SettlementAsset::USD;
        let strike = Decimal::from_str(parts[2]).unwrap_or_default();
        let expiration = NaiveDate::parse_from_str(parts[1], "%d%b%y").unwrap();

        let option_type = match parts[3] {
            "C" => OptionType::Call,
            "P" => OptionType::Put,
            _ => return None,
        };

        Some(Self::Option { underlying, settlement, strike, expiration, option_type })
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_okex_exchange() {
        let product = Product::from_okex_exhchange("BTC-USD-250221-99000-C").unwrap();
        assert_eq!(product, Product::Option {
            underlying: CryptoAsset::BTC,
            settlement: SettlementAsset::USD,
            strike: Decimal::from_str("99000").unwrap(),
            expiration: NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
            option_type: OptionType::Call,
        });
    }

    #[test]
    fn test_from_deribit_exchange() {
        let product = Product::from_deribit_exchange("BTC-21FEB25-99000-C").unwrap();
        assert_eq!(product, Product::Option {
            underlying: CryptoAsset::BTC,
            settlement: SettlementAsset::USD,
            strike: Decimal::from_str("99000").unwrap(),
            expiration: NaiveDate::from_ymd_opt(2025, 2, 21).unwrap(),
            option_type: OptionType::Call,
        });
    }
}
