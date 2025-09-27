use chrono::Utc;
use lighter_rust::{
    Account, AccountTier, ApiResponse, Order, OrderStatus, OrderType,
    Side, TimeInForce,
};
use lighter_rust::models::{Balance, OrderBook, Pagination, PriceLevel};

#[test]
fn test_account_tier_serialization() {
    let tier = AccountTier::Premium;
    let json = serde_json::to_string(&tier).unwrap();
    assert_eq!(json, "\"PREMIUM\"");

    let deserialized: AccountTier = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, AccountTier::Premium);

    let tier = AccountTier::Standard;
    let json = serde_json::to_string(&tier).unwrap();
    assert_eq!(json, "\"STANDARD\"");
}

#[test]
fn test_side_serialization() {
    let side = Side::Buy;
    let json = serde_json::to_string(&side).unwrap();
    assert_eq!(json, "\"BUY\"");

    let deserialized: Side = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, Side::Buy);

    let side = Side::Sell;
    let json = serde_json::to_string(&side).unwrap();
    assert_eq!(json, "\"SELL\"");
}

#[test]
fn test_order_type_serialization() {
    let order_type = OrderType::Market;
    let json = serde_json::to_string(&order_type).unwrap();
    assert_eq!(json, "\"MARKET\"");

    let deserialized: OrderType = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, OrderType::Market);

    let order_type = OrderType::Limit;
    let json = serde_json::to_string(&order_type).unwrap();
    assert_eq!(json, "\"LIMIT\"");
}

#[test]
fn test_order_status_serialization() {
    let status = OrderStatus::Pending;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"PENDING\"");

    let deserialized: OrderStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, OrderStatus::Pending);

    let status = OrderStatus::Filled;
    let json = serde_json::to_string(&status).unwrap();
    assert_eq!(json, "\"FILLED\"");
}

#[test]
fn test_api_response_serialization() {
    let response = ApiResponse::<String> {
        success: true,
        data: Some("test data".to_string()),
        error: None,
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: ApiResponse<String> = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.success, true);
    assert_eq!(deserialized.data, Some("test data".to_string()));
    assert_eq!(deserialized.error, None);
}

#[test]
fn test_balance_serialization() {
    let balance = Balance {
        asset: "USDC".to_string(),
        total: "1000.50".to_string(),
        available: "900.50".to_string(),
        locked: "100.00".to_string(),
    };

    let json = serde_json::to_string(&balance).unwrap();
    let deserialized: Balance = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.asset, "USDC");
    assert_eq!(deserialized.total, "1000.50");
    assert_eq!(deserialized.available, "900.50");
    assert_eq!(deserialized.locked, "100.00");
}

#[test]
fn test_order_serialization() {
    let order = Order {
        id: "order123".to_string(),
        client_order_id: Some("client123".to_string()),
        symbol: "BTC-USDC".to_string(),
        side: Side::Buy,
        order_type: OrderType::Limit,
        status: OrderStatus::Open,
        quantity: "0.1".to_string(),
        price: Some("50000".to_string()),
        stop_price: None,
        filled_quantity: "0".to_string(),
        remaining_quantity: "0.1".to_string(),
        average_fill_price: None,
        fee: None,
        time_in_force: TimeInForce::Gtc,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        expires_at: None,
    };

    let json = serde_json::to_string(&order).unwrap();
    let deserialized: Order = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.id, "order123");
    assert_eq!(deserialized.client_order_id, Some("client123".to_string()));
    assert_eq!(deserialized.symbol, "BTC-USDC");
    assert_eq!(deserialized.side, Side::Buy);
    assert_eq!(deserialized.order_type, OrderType::Limit);
    assert_eq!(deserialized.status, OrderStatus::Open);
}

#[test]
fn test_pagination_serialization() {
    let pagination = Pagination {
        page: 1,
        limit: 50,
        total: 100,
        has_next: true,
    };

    let json = serde_json::to_string(&pagination).unwrap();
    let deserialized: Pagination = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.page, 1);
    assert_eq!(deserialized.limit, 50);
    assert_eq!(deserialized.total, 100);
    assert_eq!(deserialized.has_next, true);
}

#[test]
fn test_order_book_serialization() {
    let order_book = OrderBook {
        bids: vec![
            PriceLevel {
                price: "49900".to_string(),
                quantity: "1.5".to_string(),
            },
            PriceLevel {
                price: "49800".to_string(),
                quantity: "2.0".to_string(),
            },
        ],
        asks: vec![
            PriceLevel {
                price: "50100".to_string(),
                quantity: "1.2".to_string(),
            },
            PriceLevel {
                price: "50200".to_string(),
                quantity: "1.8".to_string(),
            },
        ],
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&order_book).unwrap();
    let deserialized: OrderBook = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.bids.len(), 2);
    assert_eq!(deserialized.asks.len(), 2);
    assert_eq!(deserialized.bids[0].price, "49900");
    assert_eq!(deserialized.asks[0].price, "50100");
}

#[test]
fn test_complex_nested_serialization() {
    let json_str = r#"
    {
        "success": true,
        "data": {
            "id": "acc123",
            "address": "0x1234567890abcdef",
            "tier": "PREMIUM",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "balances": [
                {
                    "asset": "USDC",
                    "total": "10000",
                    "available": "9000",
                    "locked": "1000"
                }
            ],
            "positions": [],
            "tier_switch_allowed_at": null
        },
        "error": null,
        "timestamp": "2024-01-01T00:00:00Z"
    }
    "#;

    let response: ApiResponse<Account> = serde_json::from_str(json_str).unwrap();
    assert!(response.success);
    assert!(response.data.is_some());

    let account = response.data.unwrap();
    assert_eq!(account.id, "acc123");
    assert_eq!(account.tier, AccountTier::Premium);
    assert_eq!(account.balances.len(), 1);
    assert_eq!(account.balances[0].asset, "USDC");
}
