use super::{OrderStatus, OrderType, Side};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub client_order_id: Option<String>,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub status: OrderStatus,
    pub quantity: String,
    pub price: Option<String>,
    pub stop_price: Option<String>,
    pub filled_quantity: String,
    pub remaining_quantity: String,
    pub average_fill_price: Option<String>,
    pub fee: Option<String>,
    pub time_in_force: TimeInForce,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    Gtc, // Good Till Cancelled
    Ioc, // Immediate Or Cancel
    Fok, // Fill Or Kill
    Day, // Good For Day
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: String,
    pub price: Option<String>,
    pub stop_price: Option<String>,
    pub client_order_id: Option<String>,
    pub time_in_force: TimeInForce,
    pub post_only: Option<bool>,
    pub reduce_only: Option<bool>,
    pub signature: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelOrderRequest {
    pub order_id: Option<String>,
    pub client_order_id: Option<String>,
    pub symbol: Option<String>,
    pub signature: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: String,
    pub order_id: String,
    pub symbol: String,
    pub side: Side,
    pub quantity: String,
    pub price: String,
    pub fee: String,
    pub fee_asset: String,
    pub is_maker: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFilter {
    pub symbol: Option<String>,
    pub status: Option<OrderStatus>,
    pub side: Option<Side>,
    pub order_type: Option<OrderType>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
