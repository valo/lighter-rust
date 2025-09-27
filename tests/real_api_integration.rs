//! Real API integration tests for Lighter Rust SDK
//!
//! These tests interact with the actual Lighter testnet API.
//! Run with: LIGHTER_TEST_PRIVATE_KEY=0x... cargo test --test real_api_integration --ignored

use lighter_rust::{Config, LighterClient, OrderType, Side, TimeInForce, AccountTier};
use std::env;

/// Test configuration from environment variables
struct TestConfig {
    private_key: String,
    api_key: Option<String>,
    base_url: String,
}

impl TestConfig {
    fn from_env() -> Self {
        Self {
            private_key: env::var("LIGHTER_TEST_PRIVATE_KEY")
                .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000000000000000000000000001".to_string()),
            api_key: env::var("LIGHTER_TEST_API_KEY").ok(),
            base_url: env::var("LIGHTER_TEST_BASE_URL")
                .unwrap_or_else(|_| "https://api.testnet.lighter.xyz".to_string()),
        }
    }

    fn create_config(&self) -> lighter_rust::error::Result<Config> {
        let mut config = Config::new()
            .with_base_url(&self.base_url)?
            .with_timeout(30);

        if let Some(ref api_key) = self.api_key {
            config = config.with_api_key(api_key);
        }

        Ok(config)
    }

    async fn create_client(&self) -> Result<LighterClient, Box<dyn std::error::Error>> {
        Ok(LighterClient::new(
            self.create_config()?,
            &self.private_key,
        )?)
    }
}

// ============================================================================
// ACCOUNT TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_account() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let account = client.account().get_account().await?;

    assert!(!account.id.is_empty());
    assert!(!account.address.is_empty());

    println!("Account ID: {}", account.id);
    println!("Account address: {}", account.address);
    println!("Account tier: {:?}", account.tier);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_account_stats() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let stats = client.account().get_account_stats().await?;

    println!("Total volume: {}", stats.total_volume);
    println!("Total trades: {}", stats.total_trades);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_balances() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let balances = client.account().get_balances().await?;

    for balance in &balances {
        println!("Asset: {}, Available: {}, Locked: {}",
            balance.asset, balance.available, balance.locked);
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_can_switch_tier() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let can_switch = client.account().can_switch_tier().await?;
    println!("Can switch tier: {}", can_switch);

    Ok(())
}

// ============================================================================
// ORDER TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_create_and_cancel_order() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    // Create a limit buy order with low price
    let order = client.orders().create_order(
        "BTC-USDC",
        Side::Buy,
        OrderType::Limit,
        "0.001",
        Some("10000"),  // Low price to avoid execution
        None,
        Some(TimeInForce::Gtc),
        None,
        None,
    ).await?;

    println!("Created order: {}", order.id);
    assert_eq!(order.symbol, "BTC-USDC");

    // Cancel the order
    client.orders().cancel_order(Some(&order.id), None, None).await?;
    println!("Cancelled order: {}", order.id);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_order() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    // Create an order first
    let created = client.orders().create_order(
        "BTC-USDC",
        Side::Buy,
        OrderType::Limit,
        "0.001",
        Some("10000"),
        None,
        Some(TimeInForce::Gtc),
        None,
        None,
    ).await?;

    // Retrieve it
    let retrieved = client.orders().get_order(&created.id).await?;
    assert_eq!(retrieved.id, created.id);

    // Clean up
    let _ = client.orders().cancel_order(Some(&created.id), None, None).await;

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_orders() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let filter = lighter_rust::models::OrderFilter {
        symbol: Some("BTC-USDC".to_string()),
        status: None,
        side: None,
        order_type: None,
        start_time: None,
        end_time: None,
        page: Some(0),
        limit: Some(10),
    };

    let (orders, pagination) = client.orders().get_orders(Some(filter)).await?;

    println!("Retrieved {} orders", orders.len());
    if let Some(page_info) = pagination {
        println!("Total orders: {}", page_info.total);
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_trades() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let trades = client.orders().get_trades(Some("BTC-USDC")).await?;

    println!("Retrieved {} trades", trades.len());
    for trade in trades.iter().take(5) {
        println!("Trade: {} - Price: {}, Quantity: {}",
            trade.id, trade.price, trade.quantity);
    }

    Ok(())
}

// ============================================================================
// MARKET DATA TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_candlesticks() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let end = chrono::Utc::now();
    let start = end - chrono::Duration::hours(1);

    let candles = client.market_data().get_candlesticks(
        "BTC-USDC",
        "1m",
        start,
        end,
        Some(10),
    ).await?;

    println!("Retrieved {} candlesticks", candles.len());
    for candle in candles.iter().take(3) {
        println!("Candle - Open: {}, High: {}, Low: {}, Close: {}",
            candle.open, candle.high, candle.low, candle.close);
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_market_stats() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let stats = client.market_data().get_market_stats("BTC-USDC").await?;

    println!("Market stats for {}", stats.symbol);
    println!("Last price: {}", stats.last_price);
    println!("Volume: {}", stats.volume);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_ticker() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let ticker = client.market_data().get_ticker("BTC-USDC").await?;

    println!("Ticker for {}: Price: {}", ticker.symbol, ticker.price);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_order_book() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let order_book = client.market_data().get_order_book("BTC-USDC", Some(5)).await?;

    println!("Order book - {} bids, {} asks",
        order_book.bids.len(), order_book.asks.len());

    if let Some(best_bid) = order_book.bids.first() {
        println!("Best bid: {} @ {}", best_bid.quantity, best_bid.price);
    }

    if let Some(best_ask) = order_book.asks.first() {
        println!("Best ask: {} @ {}", best_ask.quantity, best_ask.price);
    }

    Ok(())
}

// ============================================================================
// TRANSACTION TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_latest_block() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let block = client.transactions().get_latest_block().await?;

    println!("Latest block: {} - Hash: {}", block.number, block.hash);

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_get_transactions() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let transactions = client.transactions().get_transactions(
        "all",  // Filter type
        Some(10),
        Some(0),
    ).await?;

    println!("Retrieved {} transactions", transactions.len());
    for tx in transactions.iter().take(3) {
        println!("Transaction: {} - Type: {:?}", tx.hash, tx.tx_type);
    }

    Ok(())
}

// ============================================================================
// WEBSOCKET TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_websocket_connection() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let mut client = test_config.create_client().await?;

    // Connect to WebSocket
    client.websocket().connect().await?;
    println!("WebSocket connected");

    // Subscribe to ticker
    let sub_id = client.websocket().subscribe(
        "ticker",
        Some(serde_json::json!({"symbol": "BTC-USDC"})),
    ).await?;
    println!("Subscribed with ID: {}", sub_id);

    // Receive a few messages
    for _ in 0..3 {
        if let Some(message) = client.websocket().next_message().await? {
            println!("Received message: {:?}", message);
        }
    }

    // Unsubscribe and disconnect
    client.websocket().unsubscribe(&sub_id).await?;
    client.websocket().close().await?;

    Ok(())
}

// ============================================================================
// SIGNER TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_ethereum_signer() -> Result<(), Box<dyn std::error::Error>> {
    use lighter_rust::signers::{EthereumSigner, Signer};

    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let signer = EthereumSigner::from_private_key(private_key)?;

    let address = signer.get_address()?;
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42);

    let signature = signer.sign_message("Hello, Lighter!")?;
    assert!(signature.starts_with("0x"));

    println!("Signer address: {}", address);
    println!("Signature: {}", signature);

    Ok(())
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_invalid_order_id() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    let result = client.orders().cancel_order(
        Some("invalid_order_id_12345"),
        None,
        None,
    ).await;

    assert!(result.is_err());
    println!("Expected error: {:?}", result.unwrap_err());

    Ok(())
}

// ============================================================================
// CONCURRENT OPERATIONS TEST
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_concurrent_requests() -> Result<(), Box<dyn std::error::Error>> {
    let test_config = TestConfig::from_env();
    let client = test_config.create_client().await?;

    // Make multiple requests concurrently
    let (account, ticker, order_book) = tokio::join!(
        client.account().get_account(),
        client.market_data().get_ticker("BTC-USDC"),
        client.market_data().get_order_book("BTC-USDC", Some(5)),
    );

    assert!(account.is_ok());
    assert!(ticker.is_ok());
    assert!(order_book.is_ok());

    println!("All concurrent requests succeeded");

    Ok(())
}