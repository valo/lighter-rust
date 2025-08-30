# Claude Context for Lighter Rust SDK

This document provides context and guidelines for Claude when working on the Lighter Rust SDK codebase.

## Project Overview

The Lighter Rust SDK is a comprehensive Rust client library for the Lighter trading platform (v2). It provides:
- Complete REST API coverage for trading operations
- WebSocket support for real-time market data
- Ethereum wallet integration using Alloy
- Support for both Standard and Premium account tiers

## Architecture

### Core Components

```
src/
├── api/                 # API endpoint implementations
│   ├── account.rs      # Account management operations
│   ├── order.rs        # Order placement and management
│   ├── transaction.rs  # Transaction history
│   └── candlestick.rs  # Market data and OHLCV
├── client/             # HTTP and WebSocket clients
│   ├── api_client.rs   # REST API client with retry logic
│   ├── signer_client.rs # Client with signing capabilities
│   └── ws_client.rs    # WebSocket client
├── models/             # Data models and types
│   ├── account.rs      # Account-related models
│   ├── order.rs        # Order and trade models
│   └── common.rs       # Shared types and enums
├── signers/            # Ethereum signing
│   └── ethereum.rs     # Alloy-based signer with mnemonic support
├── error.rs            # Error types and handling
├── config.rs           # Configuration management
├── nonce.rs            # Nonce generation for requests
└── lib.rs              # Main client interface
```

## Key Design Decisions

### 1. Error Handling
- Uses `thiserror` for error derivation
- Large error variants (reqwest::Error, tungstenite::Error) are boxed to reduce Result size
- Comprehensive error types for different failure modes

### 2. Async Architecture
- Fully async using Tokio runtime
- All API calls are async and return `Result<T, LighterError>`
- WebSocket operations use tokio-tungstenite

### 3. Signing Architecture
- Uses Alloy instead of ethers-rs for Ethereum operations
- Supports both private key and mnemonic phrase authentication
- Signing is abstracted through a `Signer` trait for extensibility

### 4. HTTP Client Features
- Automatic retry with exponential backoff and jitter
- Connection pooling for performance
- Rate limit awareness
- Comprehensive logging with tracing

## Dependencies

### Core Dependencies
- `alloy` (v0.5) - Ethereum signing and primitives
- `tokio` (v1.0) - Async runtime
- `reqwest` (v0.11) - HTTP client
- `tungstenite`/`tokio-tungstenite` (v0.20) - WebSocket support
- `serde`/`serde_json` - Serialization
- `tracing`/`tracing-subscriber` - Structured logging

### Cryptographic Dependencies
- `bip39` (v2.0) - Mnemonic phrase support
- `tiny-hderive` (v0.3) - HD wallet derivation
- `sha3` (v0.10) - Keccak256 hashing
- `secp256k1` (v0.29) - Elliptic curve operations

## API Patterns

### Creating Orders
```rust
client.orders().create_order(
    symbol,           // e.g., "BTC-USDC"
    side,            // Side::Buy or Side::Sell
    order_type,      // OrderType::Limit, Market, etc.
    quantity,        // Amount as string
    price,           // Optional price for limit orders
    client_order_id, // Optional client-provided ID
    time_in_force,   // Optional (defaults to GTC)
    post_only,       // Optional bool
    reduce_only,     // Optional bool
).await?
```

### Account Management
```rust
// Get account info
let account = client.account().get_account().await?;

// Switch tiers (requires no open positions)
if client.account().can_switch_tier().await? {
    client.account().change_account_tier(AccountTier::Premium).await?;
}
```

## Testing Strategy

### Unit Tests
- Located in `tests/` directory
- Cover serialization, signing, mnemonic support
- Use mockito for API mocking

### Integration Tests
- `tests/integration_tests.rs` - Mock API server tests
- Examples serve as integration tests for real usage patterns

### CI/CD
- GitHub Actions workflow (`.github/workflows/ci.yml`)
- Runs on push and PR
- Checks: fmt, clippy, tests, multi-platform builds

## Code Quality Standards

### Formatting
- Use `cargo fmt` for all code
- Line width: 100 characters (Rust default)
- Imports sorted and grouped

### Linting
- `cargo clippy` with `-D warnings` flag
- Box large error variants to avoid `result_large_err` lint
- Avoid unnecessary clones and allocations

### Documentation
- All public APIs should have doc comments
- Examples in doc comments where helpful
- Comprehensive README and integration guide

## Common Tasks

### Adding a New API Endpoint
1. Add request/response models in `src/models/`
2. Implement the endpoint in appropriate `src/api/` module
3. Add method to main client if needed
4. Write tests for serialization and API calls
5. Update documentation

### Modifying Error Types
1. Update `src/error.rs`
2. Box large variants to avoid clippy warnings
3. Update error conversions if needed
4. Fix any broken error handling in codebase

### Adding WebSocket Functionality
1. Update `src/client/ws_client.rs`
2. Add message types to models
3. Implement subscription methods
4. Handle new message types in `next_message()`
5. Add examples showing usage

## Known Issues and TODOs

1. **Warnings to Address**:
   - Ambiguous glob re-exports in lib.rs
   - Function with too many arguments in create_order
   - Some unused fields in examples

2. **Future Enhancements**:
   - Add order book depth tracking
   - Implement order modification endpoints
   - Add more comprehensive WebSocket message types
   - Create mock server for better testing

## Development Workflow

1. **Before Making Changes**:
   ```bash
   git pull origin master
   cargo fmt --all
   cargo clippy --all-targets --all-features
   ```

2. **After Making Changes**:
   ```bash
   cargo fmt --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo test
   cargo build --all-targets
   ```

3. **Commit Convention**:
   - Follow Conventional Commits specification
   - Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore

## External Resources

- [Lighter API Documentation](https://apibetadocs.lighter.xyz/docs)
- [Python SDK Reference](https://github.com/elliottech/lighter-python)
- [Alloy Documentation](https://github.com/alloy-rs/alloy)

## Notes for Claude

### When Asked to Make Changes:
1. Always run `cargo fmt` after modifications
2. Check `cargo clippy` for any new warnings
3. Update tests if changing functionality
4. Update documentation for API changes
5. Follow existing code patterns and conventions

### When Adding Features:
1. Check if similar functionality exists in Python SDK
2. Maintain consistency with existing API design
3. Add comprehensive error handling
4. Include examples showing usage
5. Update this CLAUDE.md if adding major features

### Performance Considerations:
1. Box large error types to reduce stack usage
2. Use references where possible to avoid clones
3. Leverage connection pooling in HTTP client
4. Be mindful of rate limits in examples

### Security Considerations:
1. Never log private keys or sensitive data
2. Always use secure random for nonce generation
3. Validate all user inputs
4. Use constant-time comparisons for sensitive data
5. Keep dependencies updated for security patches