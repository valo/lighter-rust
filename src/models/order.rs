use super::{OrderStatus, OrderType, Side};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    pub time_in_force: TimeInForce,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    pub signature: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub signature: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelAllOrdersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub signature: String,
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn create_order_request_serializes_to_camel_case() {
        let request = CreateOrderRequest {
            symbol: "BTC-USDC".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: "1".to_string(),
            price: Some("30000".to_string()),
            stop_price: None,
            client_order_id: Some("my-order".to_string()),
            time_in_force: TimeInForce::Gtc,
            post_only: Some(true),
            reduce_only: None,
            signature: "sig".to_string(),
            nonce: 123,
        };

        let value = serde_json::to_value(&request).expect("serialized request");
        assert_eq!(value["orderType"], json!("LIMIT"));
        assert_eq!(value["timeInForce"], json!("GTC"));
        assert_eq!(value["clientOrderId"], json!("my-order"));
        assert_eq!(value["postOnly"], json!(true));
        assert_eq!(value["nonce"], json!(123));
    }

    #[test]
    fn cancel_order_request_serializes_optional_fields() {
        let request = CancelOrderRequest {
            order_id: None,
            client_order_id: Some("cid".to_string()),
            symbol: Some("BTC-USDC".to_string()),
            signature: "sig".to_string(),
            nonce: 42,
        };

        let value = serde_json::to_value(&request).expect("serialized request");
        assert!(value.get("orderId").is_none());
        assert_eq!(value["clientOrderId"], json!("cid"));
        assert_eq!(value["symbol"], json!("BTC-USDC"));
    }

    #[test]
    fn order_deserializes_from_camel_case_payload() {
        let created_at = Utc.with_ymd_and_hms(2024, 5, 1, 0, 0, 0).unwrap();
        let updated_at = Utc.with_ymd_and_hms(2024, 5, 1, 1, 0, 0).unwrap();

        let payload = json!({
            "id": "order123",
            "clientOrderId": "cid",
            "symbol": "BTC-USDC",
            "side": "BUY",
            "orderType": "LIMIT",
            "status": "OPEN",
            "quantity": "1",
            "price": "30000",
            "stopPrice": null,
            "filledQuantity": "0",
            "remainingQuantity": "1",
            "averageFillPrice": null,
            "fee": null,
            "timeInForce": "GTC",
            "createdAt": created_at,
            "updatedAt": updated_at,
            "expiresAt": null
        });

        let order: Order = serde_json::from_value(payload).expect("deserialized order");
        assert_eq!(order.id, "order123");
        assert_eq!(order.client_order_id.as_deref(), Some("cid"));
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.status, OrderStatus::Open);
        assert_eq!(order.time_in_force, TimeInForce::Gtc);
    }
}
