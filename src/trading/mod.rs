use crate::config::Config;
use crate::error::{LighterError, Result};
use crate::metadata::{MarketInfo, MarketMetadata};
use crate::nonce::NonceManager;
use crate::signers::FFISigner;
use crate::{api::transaction_api::LighterTransactionApi, client::ApiClient};
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct LighterFfiTradingClient {
    transaction_api: LighterTransactionApi,
    metadata: MarketMetadata,
    markets: RwLock<HashMap<String, MarketInfo>>,
    nonce_manager: NonceManager,
}

#[derive(Debug)]
pub struct SubmittedOrder {
    pub order: serde_json::Value,
    pub response: crate::api::transaction_api::TxResponse,
}

impl LighterFfiTradingClient {
    pub async fn new(
        config: Config,
        private_key: &str,
        account_index: i32,
        api_key_index: i32,
    ) -> Result<Self> {
        let metadata = MarketMetadata::new(config.clone())?;
        let markets_vec = metadata.fetch_markets().await?;
        let mut markets = HashMap::with_capacity(markets_vec.len());
        for entry in markets_vec {
            let key = entry.symbol.to_uppercase();
            markets.insert(key, entry);
        }

        let api_client = ApiClient::new(config.clone())?;
        let base_url = config.base_url.clone();
        let signing_url = build_signing_url(&base_url)?;
        let signer = FFISigner::new(&signing_url, private_key, api_key_index, account_index)?;
        let transaction_api = LighterTransactionApi::with_signer(api_client, signer);

        Ok(Self {
            transaction_api,
            metadata,
            markets: RwLock::new(markets),
            nonce_manager: NonceManager::new(),
        })
    }

    pub async fn create_market_order(
        &self,
        symbol: &str,
        is_buy: bool,
        base_amount: &Decimal,
        reduce_only: bool,
    ) -> Result<SubmittedOrder> {
        let info = self.market(symbol).await?;
        let size_decimals = info.supported_size_decimals.unwrap_or(0);

        let amount = scale_decimal(base_amount, size_decimals)
            .ok_or_else(|| LighterError::Signing("unable to convert order size".to_string()))?;

        let price = 0i32;
        let is_ask = !is_buy;
        let trigger_price = 0i32;
        let order_expiry = 0i64;
        let nonce = self.nonce_manager.generate()? as i64;
        let client_order_index = nonce;

        let (order, response) = self
            .transaction_api
            .create_order(
                info.market_id,
                client_order_index,
                amount,
                price,
                is_ask,
                crate::models::common::OrderType::Market,
                crate::models::order::TimeInForce::Ioc,
                reduce_only,
                trigger_price,
                order_expiry,
                nonce,
            )
            .await?;

        Ok(SubmittedOrder { order, response })
    }

    async fn market(&self, symbol: &str) -> Result<MarketInfo> {
        let key = symbol.to_uppercase();
        if let Some(info) = self.markets.read().await.get(&key) {
            return Ok(info.clone());
        }

        let fresh = self.metadata.fetch_markets().await?;
        let mut guard = self.markets.write().await;
        for entry in fresh {
            let key = entry.symbol.to_uppercase();
            guard.insert(key, entry);
        }
        guard.get(&key).cloned().ok_or_else(|| LighterError::Api {
            status: 404,
            message: format!("Unknown Lighter market {key}"),
        })
    }
}

fn build_signing_url(url: &url::Url) -> Result<String> {
    let host = url
        .host_str()
        .ok_or_else(|| LighterError::Config("Missing host in Lighter base URL".to_string()))?;
    let mut signing = format!("{}://{}", url.scheme(), host);
    if let Some(port) = url.port() {
        signing.push(':');
        signing.push_str(&port.to_string());
    }
    Ok(signing)
}

fn scale_decimal(value: &Decimal, decimals: u32) -> Option<i64> {
    if decimals > 19 {
        return None;
    }
    let multiplier = Decimal::from(10u64.pow(decimals));
    (value * multiplier).to_i64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scales_decimal() {
        let value = Decimal::new(1234, 3); // 1.234
        let scaled = scale_decimal(&value, 3).unwrap();
        assert_eq!(scaled, 1234);
    }
}
