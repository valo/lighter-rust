use crate::client::ApiClient;
use crate::config::Config;
use crate::error::{LighterError, Result};
use crate::signers::FFISigner;
use serde::{Deserialize, Serialize};
use serde_json;

/// Lightweight client that mirrors the Python SDK behaviour by driving the
/// FFI signer directly. This avoids the Ethereum-style key requirements of the
/// default `LighterClient` while still exposing read-only account helpers.
pub struct LighterFfiClient {
    api_client: ApiClient,
    #[allow(dead_code)]
    signer: FFISigner,
    account_index: i32,
}

impl LighterFfiClient {
    /// Create a new client using the native Lighter API key material.
    pub fn new(
        config: Config,
        private_key: &str,
        account_index: i32,
        api_key_index: i32,
    ) -> Result<Self> {
        let base_url = config.base_url.clone();
        let base_url_str = {
            let mut url = format!(
                "{}://{}",
                base_url.scheme(),
                base_url.host_str().unwrap_or_default()
            );
            if let Some(port) = base_url.port() {
                url.push(':');
                url.push_str(&port.to_string());
            }
            url
        };

        let api_client = ApiClient::new(config.clone())?;
        let signer = FFISigner::new(&base_url_str, private_key, api_key_index, account_index)?;

        Ok(Self {
            api_client,
            signer,
            account_index,
        })
    }

    /// Fetch the full account payload.
    pub async fn get_account(&self) -> Result<AccountSnapshot> {
        self.fetch_account().await
    }

    /// Convenience helper returning only balances.
    pub async fn get_balances(&self) -> Result<Vec<AccountBalance>> {
        let account = self.fetch_account().await?;
        Ok(account.balances)
    }

    /// Convenience helper returning only positions.
    pub async fn get_positions(&self) -> Result<Vec<AccountPosition>> {
        let account = self.fetch_account().await?;
        Ok(account.positions)
    }

    async fn fetch_account(&self) -> Result<AccountSnapshot> {
        let endpoint = format!("/account?by=index&value={}", self.account_index);
        let value: serde_json::Value = self.api_client.get(&endpoint).await?;

        let response: RawAccountResponse = serde_json::from_value(value)?;
        let account = response
            .accounts
            .into_iter()
            .find(|entry| entry.account_index == self.account_index as i64)
            .ok_or_else(|| LighterError::Api {
                status: 404,
                message: "Account not found".to_string(),
            })?;

        Ok(account.into())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RawAccountResponse {
    #[serde(default)]
    accounts: Vec<RawAccountEntry>,
    #[serde(default)]
    _code: i32,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAccountEntry {
    account_index: i64,
    #[serde(default)]
    available_balance: String,
    #[serde(default)]
    collateral: String,
    #[serde(default)]
    cross_asset_value: String,
    #[serde(default)]
    total_asset_value: String,
    #[serde(default)]
    l1_address: String,
    #[serde(default)]
    positions: Vec<RawAccountPosition>,
    #[serde(default)]
    balances: Vec<RawAccountBalance>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAccountBalance {
    #[serde(default)]
    asset: String,
    #[serde(default)]
    total: String,
    #[serde(default)]
    available: String,
    #[serde(default)]
    locked: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawAccountPosition {
    #[serde(default)]
    symbol: String,
    #[serde(default)]
    position: String,
    #[serde(default)]
    avg_entry_price: String,
    #[serde(default)]
    position_value: String,
    #[serde(default)]
    unrealized_pnl: String,
    #[serde(default)]
    realized_pnl: String,
    #[serde(default)]
    margin_mode: i32,
    #[serde(default)]
    initial_margin_fraction: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountSnapshot {
    pub account_index: i64,
    pub l1_address: String,
    pub available_balance: String,
    pub collateral: String,
    pub cross_asset_value: String,
    pub total_asset_value: String,
    pub balances: Vec<AccountBalance>,
    pub positions: Vec<AccountPosition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountBalance {
    pub asset: String,
    pub total: String,
    pub available: String,
    pub locked: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountPosition {
    pub symbol: String,
    pub size: String,
    pub average_entry_price: String,
    pub notional_value: String,
    pub unrealized_pnl: String,
    pub realized_pnl: String,
    pub margin_mode: i32,
    pub initial_margin_fraction: String,
}

impl From<RawAccountEntry> for AccountSnapshot {
    fn from(value: RawAccountEntry) -> Self {
        let balances = if value.balances.is_empty() {
            vec![AccountBalance {
                asset: "USDC".to_string(),
                total: value.total_asset_value.clone(),
                available: value.available_balance.clone(),
                locked: String::new(),
            }]
        } else {
            value
                .balances
                .into_iter()
                .map(|balance| AccountBalance {
                    asset: balance.asset,
                    total: balance.total,
                    available: balance.available,
                    locked: balance.locked,
                })
                .collect()
        };

        let positions = value
            .positions
            .into_iter()
            .map(|position| AccountPosition {
                symbol: position.symbol,
                size: position.position,
                average_entry_price: position.avg_entry_price,
                notional_value: position.position_value,
                unrealized_pnl: position.unrealized_pnl,
                realized_pnl: position.realized_pnl,
                margin_mode: position.margin_mode,
                initial_margin_fraction: position.initial_margin_fraction,
            })
            .collect();

        Self {
            account_index: value.account_index,
            l1_address: value.l1_address,
            available_balance: value.available_balance,
            collateral: value.collateral,
            cross_asset_value: value.cross_asset_value,
            total_asset_value: value.total_asset_value,
            balances,
            positions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_account_snapshot() {
        let raw = r#"
        {
            "code": 200,
            "accounts": [
                {
                    "account_index": 70407,
                    "available_balance": "24799.622165",
                    "collateral": "20721.549363",
                    "cross_asset_value": "20721.549363",
                    "total_asset_value": "27737.715866",
                    "l1_address": "0x166ed9f7A56053c7c4E77CB0C91a9E46bbC5e8b0",
                    "balances": [
                        {"asset": "USDC", "total": "27737.715866", "available": "24799.622165", "locked": "0"}
                    ],
                    "positions": [
                        {
                            "symbol": "MEGA",
                            "position": "17963.0",
                            "avg_entry_price": "0.48774",
                            "position_value": "8815.162620",
                            "unrealized_pnl": "53.848923",
                            "realized_pnl": "0.000000",
                            "margin_mode": 1,
                            "initial_margin_fraction": "33.33"
                        }
                    ]
                }
            ]
        }
        "#;

        let response: RawAccountResponse = serde_json::from_str(raw).expect("valid json");
        let snapshot: AccountSnapshot = response.accounts.into_iter().next().unwrap().into();

        assert_eq!(snapshot.account_index, 70407);
        assert_eq!(snapshot.balances.len(), 1);
        assert_eq!(snapshot.positions.len(), 1);
        assert_eq!(snapshot.positions[0].symbol, "MEGA");
    }
}
