use lighter_rust::{AccountTier, Config, LighterClient, OrderType, Side};
use serde_json::json;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_account_integration() {
    let account_response = json!({
        "success": true,
        "data": {
            "id": "test_account",
            "address": "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1",
            "tier": "STANDARD",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "balances": [
                {
                    "asset": "USDC",
                    "total": "10000.00",
                    "available": "9000.00",
                    "locked": "1000.00"
                }
            ],
            "positions": [],
            "tier_switch_allowed_at": null
        },
        "error": null,
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/api/v1/account")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(account_response.to_string())
        .create_async()
        .await;

    let config = Config::new()
        .with_api_key("test_key")
        .with_base_url(&server.url())
        .unwrap();

    let client = LighterClient::new(
        config,
        "0000000000000000000000000000000000000000000000000000000000000001",
    )
    .unwrap();

    let account = client.account().get_account().await.unwrap();
    assert_eq!(account.id, "test_account");
    assert_eq!(account.tier, AccountTier::Standard);
    assert_eq!(account.balances.len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_create_order_integration() {
    let order_response = json!({
        "success": true,
        "data": {
            "id": "order_123",
            "client_order_id": "client_456",
            "symbol": "BTC-USDC",
            "side": "BUY",
            "order_type": "LIMIT",
            "status": "OPEN",
            "quantity": "0.1",
            "price": "45000.00",
            "filled_quantity": "0",
            "remaining_quantity": "0.1",
            "time_in_force": "GTC",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        },
        "error": null,
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/api/v1/orders")
        .match_header("authorization", "Bearer test_key")
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(order_response.to_string())
        .create_async()
        .await;

    let config = Config::new()
        .with_api_key("test_key")
        .with_base_url(&server.url())
        .unwrap();

    let client = LighterClient::new(
        config,
        "0000000000000000000000000000000000000000000000000000000000000001",
    )
    .unwrap();

    let order = client
        .orders()
        .create_order(
            "BTC-USDC",
            Side::Buy,
            OrderType::Limit,
            "0.1",
            Some("45000.00"),
            None,
            None,
            None,
            Some(false),
            Some(false),
        )
        .await
        .unwrap();

    assert_eq!(order.id, "order_123");
    assert_eq!(order.symbol, "BTC-USDC");
    assert_eq!(order.status, lighter_rust::OrderStatus::Open);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_get_market_data_integration() {
    let market_stats_response = json!({
        "success": true,
        "data": {
            "symbol": "BTC-USDC",
            "price_change": "1000.00",
            "price_change_percent": "2.22",
            "last_price": "46000.00",
            "bid_price": "45999.00",
            "ask_price": "46001.00",
            "volume": "123.45",
            "quote_volume": "5678900.00",
            "high_price": "47000.00",
            "low_price": "44000.00",
            "open_price": "45000.00",
            "timestamp": "2024-01-01T00:00:00Z"
        },
        "error": null,
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/api/v1/market/stats/BTC-USDC")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(market_stats_response.to_string())
        .create_async()
        .await;

    let config = Config::new().with_base_url(&server.url()).unwrap();

    let client = LighterClient::new_read_only(config).unwrap();

    let stats = client
        .market_data()
        .get_market_stats("BTC-USDC")
        .await
        .unwrap();
    assert_eq!(stats.symbol, "BTC-USDC");
    assert_eq!(stats.last_price, "46000.00");
    assert_eq!(stats.price_change_percent, "2.22");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_error_handling_integration() {
    let error_response = json!({
        "success": false,
        "data": null,
        "error": "Insufficient balance",
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/api/v1/orders")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(error_response.to_string())
        .create_async()
        .await;

    let config = Config::new()
        .with_api_key("test_key")
        .with_base_url(&server.url())
        .unwrap();

    let client = LighterClient::new(
        config,
        "0000000000000000000000000000000000000000000000000000000000000001",
    )
    .unwrap();

    let result = client
        .orders()
        .create_order(
            "BTC-USDC",
            Side::Buy,
            OrderType::Market,
            "100.0",
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await;

    assert!(result.is_err());
    match result.err().unwrap() {
        lighter_rust::LighterError::Api { status, message } => {
            assert_eq!(status, 400);
            assert!(message.contains("Insufficient balance"));
        }
        _ => panic!("Expected Api error with status 400"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_rate_limit_handling() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/api/v1/account")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "success": false,
                "error": "Rate limit exceeded",
                "timestamp": "2024-01-01T00:00:00Z"
            })
            .to_string(),
        )
        .create_async()
        .await;

    let config = Config::new()
        .with_api_key("test_key")
        .with_base_url(&server.url())
        .unwrap()
        .with_max_retries(0);

    let client = LighterClient::new(
        config,
        "0000000000000000000000000000000000000000000000000000000000000001",
    )
    .unwrap();

    let result = client.account().get_account().await;
    assert!(result.is_err());

    match result.err().unwrap() {
        lighter_rust::LighterError::RateLimit => {}
        _ => panic!("Expected RateLimit error"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn test_pagination_integration() {
    let orders_response = json!({
        "success": true,
        "data": {
            "orders": [
                {
                    "id": "order_1",
                    "symbol": "BTC-USDC",
                    "side": "BUY",
                    "order_type": "LIMIT",
                    "status": "FILLED",
                    "quantity": "0.1",
                    "filled_quantity": "0.1",
                    "remaining_quantity": "0",
                    "time_in_force": "GTC",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z"
                }
            ],
            "pagination": {
                "page": 1,
                "limit": 50,
                "total": 100,
                "has_next": true
            }
        },
        "error": null,
        "timestamp": "2024-01-01T00:00:00Z"
    });

    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("GET", "/api/v1/orders?page=1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(orders_response.to_string())
        .create_async()
        .await;

    let config = Config::new()
        .with_api_key("test_key")
        .with_base_url(&server.url())
        .unwrap();

    let client = LighterClient::new(
        config,
        "0000000000000000000000000000000000000000000000000000000000000001",
    )
    .unwrap();

    let filter = lighter_rust::models::OrderFilter {
        symbol: None,
        status: None,
        side: None,
        order_type: None,
        start_time: None,
        end_time: None,
        page: Some(1),
        limit: None,
    };

    let (orders, pagination) = client.orders().get_orders(Some(filter)).await.unwrap();
    assert_eq!(orders.len(), 1);
    assert!(pagination.is_some());

    let p = pagination.unwrap();
    assert_eq!(p.page, 1);
    assert_eq!(p.total, 100);
    assert!(p.has_next);
}
