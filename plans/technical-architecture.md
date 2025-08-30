# Lighter Rust SDK Technical Architecture

## Overview
The Lighter Rust SDK is designed as a comprehensive, async-first client library for interacting with the Lighter trading platform. It follows a modular architecture inspired by the Python SDK while leveraging Rust's type safety and performance characteristics.

## Architecture Principles
1. **Type Safety**: Leverage Rust's type system for compile-time guarantees
2. **Async-First**: Built on Tokio for high-performance concurrent operations
3. **Modular Design**: Separation of concerns with clear module boundaries
4. **Zero-Cost Abstractions**: Performance without sacrificing usability
5. **Error Resilience**: Comprehensive error handling with recovery strategies

## Core Components

### 1. Client Layer
```
┌─────────────────────────────────────────┐
│           LighterClient                  │
│  (High-level unified interface)         │
├─────────────────────────────────────────┤
│  - account(): AccountApi                │
│  - orders(): OrderApi                   │
│  - transactions(): TransactionApi       │
│  - market_data(): CandlestickApi       │
│  - websocket(): WebSocketClient        │
└─────────────────────────────────────────┘
                    │
        ┌──────────┴──────────┐
        ▼                      ▼
┌──────────────┐      ┌──────────────┐
│ SignerClient │      │ WebSocketClient│
│ (Auth layer) │      │ (Streaming)    │
└──────────────┘      └──────────────┘
        │
        ▼
┌──────────────┐
│  ApiClient   │
│ (HTTP layer) │
└──────────────┘
```

### 2. Module Structure

```
src/
├── lib.rs                 # Main entry point, public API
├── error.rs              # Error types and Result alias
├── config.rs             # Configuration management
├── nonce.rs              # Nonce generation for auth
│
├── client/               # Client implementations
│   ├── api_client.rs     # HTTP REST client
│   ├── ws_client.rs      # WebSocket client
│   └── signer_client.rs  # Authenticated client wrapper
│
├── api/                  # API endpoint implementations
│   ├── account.rs        # Account management
│   ├── order.rs          # Trading operations
│   ├── transaction.rs    # Transaction queries
│   └── candlestick.rs    # Market data
│
├── models/               # Data structures
│   ├── common.rs         # Shared types and enums
│   ├── account.rs        # Account-related models
│   └── order.rs          # Order and trade models
│
└── signers/              # Cryptographic signing
    └── ethereum.rs       # Ethereum/Alloy integration
```

## Component Details

### 1. Error Handling (`error.rs`)
- **Purpose**: Centralized error management
- **Design**: Using `thiserror` for ergonomic error types
- **Error Types**:
  - HTTP errors (network, timeouts)
  - API errors (auth, rate limits, validation)
  - Signing errors (invalid keys, signatures)
  - WebSocket errors (connection, protocol)
- **Pattern**: Result<T> type alias for consistency

### 2. Configuration (`config.rs`)
- **Purpose**: Flexible configuration management
- **Design**: Builder pattern for easy customization
- **Features**:
  - Environment-based configuration
  - Endpoint URL management
  - Timeout and retry settings
  - API key storage

### 3. HTTP Client (`client/api_client.rs`)
- **Purpose**: Low-level HTTP communication
- **Technology**: `reqwest` with async support
- **Features**:
  - Automatic JSON serialization/deserialization
  - Request/response interceptors
  - Error response handling
  - Retry logic with backoff
  - Rate limiting support

### 4. Authentication (`client/signer_client.rs`)
- **Purpose**: Request signing and authentication
- **Design**: Wrapper around ApiClient
- **Features**:
  - Automatic request signing
  - Nonce management
  - Private key handling
  - Signature generation

### 5. WebSocket Client (`client/ws_client.rs`)
- **Purpose**: Real-time data streaming
- **Technology**: `tokio-tungstenite`
- **Features**:
  - Connection management
  - Subscription handling
  - Message routing
  - Automatic reconnection
  - Heartbeat/ping-pong

### 6. Signing System (`signers/`)
- **Purpose**: Cryptographic operations
- **Technology**: Alloy for Ethereum compatibility
- **Features**:
  - EIP-191 message signing
  - Private key management
  - Address derivation
  - Signature verification

### 7. API Modules (`api/`)
Each module follows a consistent pattern:
- Struct wrapping SignerClient
- Methods for each endpoint
- Request building and signing
- Response parsing and validation
- Error handling and retries

#### Account API
- Account information retrieval
- Tier management (Standard/Premium)
- Balance and position queries
- Statistics and analytics

#### Order API
- Order placement (market, limit, stop)
- Order cancellation (single/bulk)
- Order queries with filters
- Trade history

#### Transaction API
- Transaction details
- Block information
- Confirmation waiting
- History queries

#### Market Data API
- Candlestick/OHLCV data
- Ticker information
- Order book depth
- Market statistics

### 8. Data Models (`models/`)
- **Design**: Strongly typed with serde
- **Pattern**: Domain-driven design
- **Features**:
  - Automatic serialization/deserialization
  - Type-safe enums for states
  - Optional fields for flexibility
  - Validation at type level

## Async Architecture

### Tokio Runtime
- **Single runtime**: Shared across all components
- **Task spawning**: For concurrent operations
- **Channels**: For component communication
- **Timeouts**: Built into all operations

### Concurrency Patterns
```rust
// Parallel request execution
let (account, orders, positions) = tokio::try_join!(
    client.account().get_account(),
    client.orders().get_orders(None),
    client.account().get_positions()
)?;

// Stream processing
let mut ws = client.websocket();
while let Some(msg) = ws.next_message().await? {
    process_message(msg).await?;
}
```

## Security Considerations

### Private Key Management
- Never logged or serialized
- Stored in Arc for safe sharing
- Zeroed on drop (via zeroize)
- No persistence to disk

### Request Signing
- EIP-191 compliant
- Nonce prevents replay attacks
- Timestamp validation
- Signature verification

### Network Security
- TLS for all connections
- Certificate validation
- No hardcoded credentials
- Secure random generation

## Performance Optimizations

### Connection Pooling
- Reuse HTTP connections
- Keep-alive for WebSockets
- Connection limits per host

### Serialization
- Zero-copy where possible
- Lazy deserialization
- Efficient JSON handling

### Memory Management
- Arc/Rc for shared ownership
- Minimal allocations
- Stack-allocated small strings

## Testing Strategy

### Unit Tests
- Module isolation
- Mock dependencies
- Property-based testing
- Fuzzing for parsers

### Integration Tests
- Mock server endpoints
- End-to-end workflows
- Error scenarios
- Performance benchmarks

## Comparison with Python SDK

### Similarities
- Same API coverage
- Identical endpoint structure
- Compatible data models
- Similar error handling

### Improvements in Rust
- Type safety at compile time
- Better performance (10-100x)
- Lower memory usage
- No GIL limitations
- Native async support

### Trade-offs
- Longer compilation times
- Steeper learning curve
- Less dynamic flexibility
- More verbose error handling

## Future Enhancements

### Performance
- SIMD for data processing
- Custom allocators
- Zero-copy parsing
- Request batching

### Features
- gRPC support
- GraphQL client
- Local order book
- Strategy backtesting

### Integrations
- Prometheus metrics
- OpenTelemetry tracing
- Redis caching
- Message queue support

## Dependencies

### Core Dependencies
- `tokio`: Async runtime
- `reqwest`: HTTP client
- `serde`: Serialization
- `alloy`: Ethereum operations
- `tungstenite`: WebSocket

### Development Dependencies
- `mockito`: HTTP mocking
- `tokio-test`: Async testing
- `criterion`: Benchmarking
- `proptest`: Property testing

## Deployment Considerations

### Binary Distribution
- Static linking for portability
- Cross-compilation support
- Docker images
- Package managers (cargo, apt, brew)

### Configuration
- Environment variables
- Configuration files
- Command-line arguments
- Runtime updates

### Monitoring
- Structured logging
- Metrics export
- Health checks
- Performance profiling