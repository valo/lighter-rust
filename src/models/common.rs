use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    Buy,
    Sell,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    Market,
    Limit,
    StopLoss,
    TakeProfit,
}

impl OrderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Market => "MARKET",
            Self::Limit => "LIMIT",
            Self::StopLoss => "STOP_LOSS",
            Self::TakeProfit => "TAKE_PROFIT",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Open => "OPEN",
            Self::PartiallyFilled => "PARTIALLY_FILLED",
            Self::Filled => "FILLED",
            Self::Cancelled => "CANCELLED",
            Self::Rejected => "REJECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountTier {
    Standard,
    Premium,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub has_next: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: String,
    pub quantity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn side_as_str_matches_api_format() {
        assert_eq!(Side::Buy.as_str(), "BUY");
        assert_eq!(Side::Sell.as_str(), "SELL");
    }

    #[test]
    fn order_type_as_str_matches_api_format() {
        assert_eq!(OrderType::Market.as_str(), "MARKET");
        assert_eq!(OrderType::Limit.as_str(), "LIMIT");
        assert_eq!(OrderType::StopLoss.as_str(), "STOP_LOSS");
        assert_eq!(OrderType::TakeProfit.as_str(), "TAKE_PROFIT");
    }

    #[test]
    fn order_status_as_str_matches_api_format() {
        assert_eq!(OrderStatus::Pending.as_str(), "PENDING");
        assert_eq!(OrderStatus::Open.as_str(), "OPEN");
        assert_eq!(OrderStatus::PartiallyFilled.as_str(), "PARTIALLY_FILLED");
        assert_eq!(OrderStatus::Filled.as_str(), "FILLED");
        assert_eq!(OrderStatus::Cancelled.as_str(), "CANCELLED");
        assert_eq!(OrderStatus::Rejected.as_str(), "REJECTED");
    }
}
