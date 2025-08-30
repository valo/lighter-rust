# Lighter Rust SDK Documentation

Comprehensive documentation for the Lighter Rust SDK.

## API Documentation

### Core APIs
- [AccountApi](AccountApi.md) - Account management and information
- [OrderApi](OrderApi.md) - Order placement and management
- [CandlestickApi](CandlestickApi.md) - Market data and analytics
- [TransactionApi](TransactionApi.md) - Transaction queries and monitoring

### WebSocket
- [WebSocketClient](WebSocketClient.md) - Real-time data streaming

## Model Documentation

### Account Models
- [Account](Account.md) - Account information structure
- [Balance](Balance.md) - Asset balance details
- [Position](Position.md) - Open position information
- [AccountTier](AccountTier.md) - Account tier types

### Order Models
- [Order](Order.md) - Order structure and lifecycle
- [Trade](Trade.md) - Executed trade details
- [OrderType](OrderType.md) - Order type enumeration
- [OrderStatus](OrderStatus.md) - Order status states
- [Side](Side.md) - Buy/Sell side
- [TimeInForce](TimeInForce.md) - Order time constraints

### Market Data Models
- [Candlestick](Candlestick.md) - OHLCV candlestick data
- [OrderBook](OrderBook.md) - Market depth information
- [Ticker](Ticker.md) - Current price ticker
- [MarketStats](MarketStats.md) - 24-hour market statistics

### Common Models
- [ApiResponse](ApiResponse.md) - Standard API response wrapper
- [Pagination](Pagination.md) - Pagination information

## Quick Start Guide

### Installation

```toml
[dependencies]
lighter-rust = { git = "https://github.com/yongkangc/lighter-rust" }
```

### Basic Usage

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure the client
    let config = Config::new()
        .with_api_key("your-api-key")
        .with_base_url("https://api.lighter.xyz")?;
    
    // Initialize with private key for trading
    let client = LighterClient::new(config, "your-private-key")?;
    
    // Get account information
    let account = client.account().get_account().await?;
    println!("Account: {:?}", account);
    
    Ok(())
}
```

### Read-Only Mode

For market data without trading:

```rust
let config = Config::new();
let client = LighterClient::new_read_only(config)?;

let ticker = client.market_data().get_ticker("BTC-USDC").await?;
println!("Price: {}", ticker.price);
```

## Authentication

The SDK supports two authentication methods:

### API Key Authentication
Used for read operations and account information:
```rust
let config = Config::new().with_api_key("your-api-key");
```

### Signature Authentication
Required for trading operations using Ethereum signing:
```rust
let client = LighterClient::new(config, "your-private-key")?;
```

## Error Handling

All SDK methods return `Result<T, LighterError>`:

```rust
use lighter_rust::LighterError;

match client.orders().create_order(...).await {
    Ok(order) => println!("Order created: {}", order.id),
    Err(LighterError::RateLimit) => {
        // Handle rate limiting
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Err(LighterError::Api { status, message }) => {
        eprintln!("API error {}: {}", status, message);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Rate Limiting

The SDK respects API rate limits:
- Standard tier: 100 requests/second
- Premium tier: 1000 requests/second

Implement exponential backoff for rate limit errors:

```rust
async fn with_retry<T, F, Fut>(f: F) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut retries = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(LighterError::RateLimit) if retries < 3 => {
                let delay = Duration::from_millis(100 * 2_u64.pow(retries));
                tokio::time::sleep(delay).await;
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## WebSocket Streaming

For real-time data:

```rust
let mut ws = client.websocket();
ws.connect().await?;

let sub_id = ws.subscribe("orderbook", Some(json!({
    "symbol": "BTC-USDC",
    "depth": 10
}))).await?;

while let Some(msg) = ws.next_message().await? {
    println!("Received: {:?}", msg);
}
```

## Examples

- [Basic Usage](../examples/basic_usage.rs) - Common operations
- [WebSocket Example](../examples/websocket_example.rs) - Real-time streaming
- [Trading Bot](../examples/trading_bot.rs) - Automated trading strategy

## Support

- GitHub Issues: [https://github.com/yongkangc/lighter-rust/issues](https://github.com/yongkangc/lighter-rust/issues)
- API Documentation: [https://apibetadocs.lighter.xyz/docs](https://apibetadocs.lighter.xyz/docs)

## License

MIT License - see [LICENSE](../LICENSE) file for details.