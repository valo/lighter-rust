use crate::client::ApiClient;
use crate::config::Config;
use crate::error::{LighterError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct MarketMetadata {
    api_client: ApiClient,
}

impl MarketMetadata {
    pub fn new(config: Config) -> Result<Self> {
        let api_client = ApiClient::new(config)?;
        Ok(Self { api_client })
    }

    pub async fn fetch_markets(&self) -> Result<Vec<MarketInfo>> {
        let response: OrderBooksResponse = self.api_client.get("orderBooks").await?;

        Ok(response.order_books)
    }

    pub async fn build_index_map(&self) -> Result<HashMap<String, i32>> {
        let markets = self.fetch_markets().await?;
        let mut map = HashMap::with_capacity(markets.len());
        for entry in markets {
            map.insert(entry.symbol.clone(), entry.market_id);
        }
        Ok(map)
    }

    pub async fn market_info<S: AsRef<str>>(&self, symbol: S) -> Result<MarketInfo> {
        let needle = symbol.as_ref().to_uppercase();
        let markets = self.fetch_markets().await?;
        markets
            .into_iter()
            .find(|entry| entry.symbol.eq_ignore_ascii_case(&needle))
            .ok_or_else(|| LighterError::Api {
                status: 404,
                message: format!("Unknown Lighter market: {needle}"),
            })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct OrderBooksResponse {
    pub order_books: Vec<MarketInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketInfo {
    pub market_id: i32,
    pub symbol: String,
    #[serde(default)]
    pub supported_size_decimals: Option<u32>,
    #[serde(default)]
    pub supported_price_decimals: Option<u32>,
    #[serde(default)]
    pub supported_quote_decimals: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn market_info_deserializes() {
        let payload = r#"{"market_id": 94, "symbol": "MEGA", "supported_size_decimals": 3}"#;
        let market: MarketInfo = serde_json::from_str(payload).expect("valid json");
        assert_eq!(market.market_id, 94);
        assert_eq!(market.symbol, "MEGA");
        assert_eq!(market.supported_size_decimals, Some(3));

        let serialized = serde_json::to_value(&market).expect("serialize");
        assert_eq!(serialized["market_id"], 94);
        assert_eq!(serialized["symbol"], "MEGA");
        assert_eq!(serialized["supported_size_decimals"], 3);
    }
}
