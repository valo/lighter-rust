# Lighter Rust SDK Integration Guide

This guide will walk you through integrating the Lighter Rust SDK into your trading application, from initial setup to advanced features.

## Table of Contents
1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [Authentication Methods](#authentication-methods)
4. [Core Concepts](#core-concepts)
5. [API Usage Patterns](#api-usage-patterns)
6. [WebSocket Integration](#websocket-integration)
7. [Error Handling](#error-handling)
8. [Best Practices](#best-practices)
9. [Migration from Python SDK](#migration-from-python-sdk)

## Installation

Add the Lighter SDK to your `Cargo.toml`:

```toml
[dependencies]
lighter-rust = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

Or use the latest development version from GitHub:

```toml
[dependencies]
lighter-rust = { git = "https://github.com/yongkangc/lighter-rust" }
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

### Basic Setup

```rust
use lighter_rust::{LighterClient, Config, init_logging};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();
    
    // Configure the client
    let config = Config::new()
        .with_api_key("your-api-key")
        .with_timeout(30);
    
    // Create client with private key
    let client = LighterClient::new(config, "your-private-key-hex")?;
    
    // Fetch account information
    let account = client.account().get_account().await?;
    println!("Account ID: {}", account.id);
    
    Ok(())
}
```

## Authentication Methods

### 1. Private Key Authentication

The most common method using a hex-encoded private key:

```rust
let client = LighterClient::new(config, "0xYOUR_PRIVATE_KEY_HEX")?;
```

### 2. Mnemonic Phrase Authentication

Use a BIP39 mnemonic phrase for HD wallet support:

```rust
// Using mnemonic with account index 0
let client = LighterClient::from_mnemonic(
    config, 
    "your twelve word mnemonic phrase here...",
    0  // account index
)?;

// Using different account indices for multiple accounts
let client1 = LighterClient::from_mnemonic(config.clone(), mnemonic, 0)?;
let client2 = LighterClient::from_mnemonic(config.clone(), mnemonic, 1)?;
```

### 3. Read-Only Mode

For market data and public endpoints only:

```rust
let client = LighterClient::new_read_only(config)?;
// Can fetch market data but cannot place orders
```

### 4. Environment-Based Configuration

Best practice for production:

```rust
use std::env;

let api_key = env::var("LIGHTER_API_KEY")?;
let private_key = env::var("LIGHTER_PRIVATE_KEY")?;

let config = Config::new()
    .with_api_key(&api_key)
    .with_base_url(&env::var("LIGHTER_API_URL").unwrap_or_else(|_| 
        "https://api.lighter.xyz/api".to_string()
    ));

let client = LighterClient::new(config, &private_key)?;
```

## Core Concepts

### Account Tiers

Lighter supports two account tiers with different features:

```rust
use lighter_rust::{AccountTier, AccountApi};

// Check current tier
let account = client.account().get_account().await?;
match account.tier {
    AccountTier::Standard => println!("Standard tier - Basic features"),
    AccountTier::Premium => println!("Premium tier - Advanced features"),
}

// Switch tiers (requires no open positions)
if client.account().can_switch_tier().await? {
    client.account().change_account_tier(AccountTier::Premium).await?;
}
```

### Order Types

```rust
use lighter_rust::{OrderType, Side, TimeInForce};

// Market order
client.orders().create_order(
    "BTC-USDC",
    Side::Buy,
    OrderType::Market,
    "0.1",
    None,  // No price for market orders
    None,  // No stop price for market orders
    None,
    None,
    None,
    None,
).await?;

// Limit order with post-only
client.orders().create_order(
    "ETH-USDC",
    Side::Sell,
    OrderType::Limit,
    "1.0",
    Some("3000.50"),
    None,
    None,
    Some(TimeInForce::Gtc),
    Some(true),  // post_only
    None,
).await?;
```

## API Usage Patterns

### Account Management

```rust
// Get account balances
let balances = client.account().get_balances().await?;
for balance in balances {
    println!("{}: {} (available: {})", 
        balance.currency, balance.total, balance.available);
}

// Get open positions
let positions = client.account().get_positions().await?;
for position in positions {
    println!("{}: {} @ avg price {}", 
        position.symbol, position.quantity, position.average_price);
}

// Get account statistics
let stats = client.account().get_account_stats().await?;
println!("Total volume: {}", stats.total_volume);
```

### Order Management

```rust
use lighter_rust::{OrderFilter, OrderStatus};

// Place an order
let order = client.orders().create_order(
    "BTC-USDC",
    Side::Buy,
    OrderType::Limit,
    "0.01",
    Some("45000"),
    None,
    None,
    Some(TimeInForce::Gtc),
    None,
    None,
).await?;

// Query order status
let order_status = client.orders().get_order(&order.id).await?;
println!("Order status: {:?}", order_status.status);

// Cancel order
client.orders().cancel_order(Some(&order.id), None, None).await?;

// Get all open orders
let filter = OrderFilter {
    status: Some(OrderStatus::Open),
    symbol: Some("BTC-USDC".to_string()),
    ..Default::default()
};
let (orders, pagination) = client.orders().get_orders(Some(filter)).await?;
```

### Market Data

```rust
use lighter_rust::{CandlestickInterval};
use chrono::{Utc, Duration};

// Get recent candlesticks
let end_time = Utc::now();
let start_time = end_time - Duration::hours(24);

let candles = client.market_data().get_candlesticks(
    "BTC-USDC",
    CandlestickInterval::OneHour,
    start_time,
    end_time,
    Some(24),
).await?;

for candle in candles {
    println!("Time: {}, Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
        candle.timestamp, candle.open, candle.high, 
        candle.low, candle.close, candle.volume);
}
```

## WebSocket Integration

### Real-time Market Data

```rust
use lighter_rust::{WebSocketClient, WsMessage};
use futures::StreamExt;

let mut ws_client = client.websocket();

// Connect to WebSocket
ws_client.connect().await?;

// Subscribe to order book updates
ws_client.subscribe_order_book("BTC-USDC").await?;

// Subscribe to trades
ws_client.subscribe_trades("ETH-USDC").await?;

// Process messages
while let Some(message) = ws_client.stream.as_mut().unwrap().next().await {
    match message? {
        WsMessage::OrderBook(book) => {
            println!("Order book update for {}", book.symbol);
        },
        WsMessage::Trade(trade) => {
            println!("Trade: {} {} @ {}", 
                trade.quantity, trade.symbol, trade.price);
        },
        WsMessage::Error(e) => {
            eprintln!("WebSocket error: {}", e);
        },
        _ => {}
    }
}
```

### Private Account Updates

```rust
// Subscribe to account updates
ws_client.subscribe_account_updates().await?;

// Subscribe to order updates
ws_client.subscribe_order_updates().await?;

// Process private messages
while let Some(message) = ws_client.stream.as_mut().unwrap().next().await {
    match message? {
        WsMessage::OrderUpdate(order) => {
            println!("Order {} status: {:?}", order.id, order.status);
        },
        WsMessage::BalanceUpdate(balance) => {
            println!("Balance update: {} {}", balance.available, balance.currency);
        },
        _ => {}
    }
}
```

## Error Handling

### Comprehensive Error Types

```rust
use lighter_rust::{LighterError, Result};

match client.orders().create_order(...).await {
    Ok(order) => println!("Order placed: {}", order.id),
    Err(e) => match e {
        LighterError::RateLimit => {
            println!("Rate limited, waiting before retry...");
            tokio::time::sleep(Duration::from_secs(1)).await;
        },
        LighterError::InsufficientBalance(msg) => {
            println!("Insufficient balance: {}", msg);
        },
        LighterError::Api { status, message } => {
            println!("API error {}: {}", status, message);
        },
        LighterError::Network(e) => {
            println!("Network error: {}", e);
        },
        _ => println!("Unexpected error: {}", e),
    }
}
```

### Retry Logic

The SDK includes built-in retry logic with exponential backoff:

```rust
let config = Config::new()
    .with_api_key("your-api-key")
    .with_max_retries(3)  // Automatic retries
    .with_timeout(30);
```

## Best Practices

### 1. Connection Pooling

The SDK automatically manages connection pooling for optimal performance:

```rust
// Connections are reused automatically
for symbol in ["BTC-USDC", "ETH-USDC", "SOL-USDC"] {
    let orders = client.orders().get_orders_for_symbol(symbol).await?;
}
```

### 2. Rate Limiting

Implement rate limiting awareness:

```rust
use tokio::time::{sleep, Duration};

// Batch operations with delays
for order_id in order_ids {
    client.orders().cancel_order(Some(&order_id), None, None).await?;
    sleep(Duration::from_millis(100)).await;  // Prevent rate limiting
}
```

### 3. Logging and Monitoring

Use structured logging for production:

```rust
use lighter_rust::init_logging_with_filter;

// Set custom log levels
init_logging_with_filter("lighter_rust=debug,my_app=info");

// Logs will include:
// - API requests and responses (debug level)
// - WebSocket messages (trace level)
// - Errors and warnings
```

### 4. Graceful Shutdown

Implement proper cleanup:

```rust
use tokio::signal;

// Graceful shutdown handler
let shutdown = signal::ctrl_c();

tokio::select! {
    _ = shutdown => {
        println!("Shutting down...");
        
        // Cancel all open orders
        client.orders().cancel_all_orders(None).await?;
        
        // Close WebSocket connections
        if ws_client.is_connected() {
            ws_client.disconnect().await?;
        }
    }
}
```

### 5. Order Management Patterns

Implement robust order management:

```rust
// Use client order IDs for idempotency
let client_order_id = uuid::Uuid::new_v4().to_string();

let order = client.orders().create_order(
    "BTC-USDC",
    Side::Buy,
    OrderType::Limit,
    "0.1",
    Some("45000"),
    None,
    Some(&client_order_id),
    Some(TimeInForce::Gtc),
    None,
    None,
).await?;

// Track order lifecycle
loop {
    let status = client.orders().get_order(&order.id).await?;
    match status.status {
        OrderStatus::Filled => {
            println!("Order filled!");
            break;
        },
        OrderStatus::Cancelled | OrderStatus::Rejected => {
            println!("Order ended: {:?}", status.status);
            break;
        },
        _ => {
            sleep(Duration::from_secs(1)).await;
        }
    }
}
```

## Migration from Python SDK

If you're migrating from the Python SDK, here's a comparison:

### Python
```python
from lighter import LighterClient

client = LighterClient(api_key="key", private_key="0x...")
account = client.get_account()
order = client.create_order(
    symbol="BTC-USDC",
    side="buy",
    order_type="limit",
    quantity="0.1",
    price="45000"
)
```

### Rust
```rust
use lighter_rust::{LighterClient, Config, Side, OrderType};

let config = Config::new().with_api_key("key");
let client = LighterClient::new(config, "0x...")?;
let account = client.account().get_account().await?;
let order = client.orders().create_order(
    "BTC-USDC",
    Side::Buy,
    OrderType::Limit,
    "0.1",
    Some("45000"),
    None,
    None,
    None,
    None,
    None,
).await?;
```

### Key Differences

1. **Async/Await**: Rust SDK is fully async, use `.await` for all API calls
2. **Error Handling**: Rust uses `Result<T, E>` for explicit error handling
3. **Type Safety**: All parameters are strongly typed enums
4. **Memory Management**: Automatic with Rust's ownership system
5. **Performance**: Generally faster with lower latency

## Advanced Features

### Grid Trading Implementation

See `examples/advanced_order_management.rs` for a complete grid trading implementation.

### Custom Signing

Implement custom signing logic:

```rust
use lighter_rust::signers::Signer;

#[derive(Debug)]
struct CustomSigner {
    // Your implementation
}

impl Signer for CustomSigner {
    fn sign_message(&self, message: &str) -> Result<String> {
        // Custom signing logic
    }
    
    fn get_address(&self) -> Result<String> {
        // Return address
    }
}
```

### Performance Optimization

For high-frequency trading:

```rust
// Pre-compile order templates
let order_template = OrderTemplate {
    symbol: "BTC-USDC".to_string(),
    side: Side::Buy,
    order_type: OrderType::Limit,
    time_in_force: TimeInForce::Ioc,
    post_only: false,
};

// Reuse for faster order placement
for price in price_levels {
    let order = order_template.clone()
        .with_price(price)
        .with_quantity(calculate_quantity(price));
    client.orders().place_templated_order(order).await?;
}
```

## Support and Resources

- **Crate**: [crates.io/crates/lighter-rust](https://crates.io/crates/lighter-rust)
- **Documentation**: [docs.rs/lighter-rust](https://docs.rs/lighter-rust)
- **API Reference**: [Lighter API Docs](https://apibetadocs.lighter.xyz/)
- **GitHub**: [github.com/yongkangc/lighter-rust](https://github.com/yongkangc/lighter-rust)
- **Examples**: See the [examples/](https://github.com/yongkangc/lighter-rust/tree/master/examples) directory for complete working examples
- **Issues**: Report bugs on [GitHub Issues](https://github.com/yongkangc/lighter-rust/issues)

## Next Steps

1. Explore the [examples](../examples/) directory for working code
2. Read the [API documentation](./README.md) for detailed method references
3. Join the Lighter community for support and updates
4. Build your trading application!