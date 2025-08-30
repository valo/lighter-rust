# Lighter Rust SDK

[![Crates.io](https://img.shields.io/crates/v/lighter-rust.svg)](https://crates.io/crates/lighter-rust)
[![Documentation](https://docs.rs/lighter-rust/badge.svg)](https://docs.rs/lighter-rust)
[![CI](https://github.com/yongkangc/lighter-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/yongkangc/lighter-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust SDK for [Lighter](https://lighter.xyz/) (v2)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lighter-rust = "0.1.0"
```

Or use the latest from GitHub:

```toml
[dependencies]
lighter-rust = { git = "https://github.com/yongkangc/lighter-rust" }
```

## Quick Start

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = Config::new()
        .with_api_key("your-api-key")
        .with_base_url("https://api.lighter.xyz")?;

    // Initialize client with private key for trading
    let client = LighterClient::new(config, "your-private-key")?;
    
    // Get account info
    let account = client.account().get_account().await?;
    println!("Account: {:?}", account);
    
    // Get market data
    let ticker = client.market_data().get_ticker("BTC-USDC").await?;
    println!("BTC-USDC Price: {}", ticker.price);
    
    Ok(())
}
```

## Features

- ✅ Complete REST API coverage
- ✅ WebSocket support for real-time data
- ✅ Ethereum wallet integration using Alloy
- ✅ Async/await with Tokio
- ✅ Type-safe API with comprehensive error handling
- ✅ Support for all order types (Market, Limit, Stop)
- ✅ Account tier management (Standard/Premium)

## Examples

Check out the [examples](./examples) directory for comprehensive examples:

- [**basic_usage.rs**](./examples/basic_usage.rs) - Basic API operations and getting started
- [**websocket_example.rs**](./examples/websocket_example.rs) - Real-time WebSocket streaming
- [**trading_bot.rs**](./examples/trading_bot.rs) - Simple trading bot with SMA strategy
- [**advanced_order_management.rs**](./examples/advanced_order_management.rs) - Grid trading, stop-loss/take-profit
- [**mnemonic_wallet.rs**](./examples/mnemonic_wallet.rs) - Using mnemonic phrases and HD wallets

Run examples with:
```bash
cargo run --example basic_usage
cargo run --example websocket_example
cargo run --example trading_bot
```

## Documentation

### API Documentation

- [**AccountApi**](./docs/AccountApi.md) - Account management operations
- [**OrderApi**](./docs/OrderApi.md) - Order placement and management
- [**TransactionApi**](./docs/TransactionApi.md) - Transaction history and tracking
- [**CandlestickApi**](./docs/CandlestickApi.md) - Market data and OHLCV
- [**WebSocketClient**](./docs/WebSocketClient.md) - Real-time data streaming

### Guides

- [**Integration Guide**](./docs/IntegrationGuide.md) - Complete integration walkthrough
- [**API Reference**](./docs/README.md) - Full API method reference

## API Coverage

### Account Management
- Get account information
- Get account statistics  
- Change account tier
- Get balances and positions

### Trading Operations
- Create orders (market, limit, stop-loss, take-profit)
- Cancel orders (single or all)
- Get order history
- Get trade history

### Market Data
- Candlestick/OHLCV data
- Tickers
- Order book depth
- Market statistics

### WebSocket Streams
- Order book updates
- Trade streams
- Account updates

## Architecture

The SDK is built with a modular architecture:

```
lighter-rust/
├── src/
│   ├── client/          # HTTP and WebSocket clients
│   ├── api/             # API endpoint implementations
│   ├── models/          # Data models and types
│   ├── signers/         # Ethereum signing (Alloy)
│   └── error.rs         # Error handling
```

## Requirements

- Rust 1.70+
- Tokio runtime

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with examples
cargo run --example basic_usage

# Build documentation
cargo doc --open
```

## Configuration

The SDK can be configured through the `Config` struct:

```rust
use lighter_rust::Config;

let config = Config::new()
    .with_api_key("your-api-key")
    .with_base_url("https://api.lighter.xyz")?
    .with_ws_url("wss://ws.lighter.xyz")?
    .with_timeout(30)
    .with_max_retries(3);
```

## Error Handling

All methods return a `Result<T, LighterError>` with comprehensive error types:

```rust
use lighter_rust::LighterError;

match client.orders().create_order(...).await {
    Ok(order) => println!("Order created: {}", order.id),
    Err(LighterError::RateLimit) => println!("Rate limited, please retry"),
    Err(LighterError::Auth(msg)) => println!("Authentication failed: {}", msg),
    Err(e) => println!("Error: {}", e),
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Disclaimer

This is an unofficial SDK. Use at your own risk. Always test thoroughly before using in production.

## Support

For issues and questions:
- Open an issue on [GitHub](https://github.com/yongkangc/lighter-rust/issues)
- Check the [API documentation](https://apibetadocs.lighter.xyz/docs)

## Related

- [Lighter Python SDK](https://github.com/elliottech/lighter-python)
- [Lighter API Docs](https://apibetadocs.lighter.xyz/docs)