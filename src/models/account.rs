use super::AccountTier;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub address: String,
    pub tier: AccountTier,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub balances: Vec<Balance>,
    pub positions: Vec<Position>,
    pub tier_switch_allowed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub total: String,
    pub available: String,
    pub locked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub side: super::Side,
    pub size: String,
    pub entry_price: String,
    pub mark_price: String,
    pub unrealized_pnl: String,
    pub margin_type: MarginType,
    pub leverage: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarginType {
    Cross,
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTierSwitchRequest {
    pub target_tier: AccountTier,
    pub signature: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStats {
    pub total_volume: String,
    pub maker_volume: String,
    pub taker_volume: String,
    pub total_fees_paid: String,
    pub total_trades: u64,
    pub win_rate: String,
    pub pnl: String,
}
