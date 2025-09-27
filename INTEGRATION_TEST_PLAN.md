# Lighter Rust SDK Integration Test Plan

## Overview
Comprehensive integration tests for the Lighter Rust SDK, testing against the real API (testnet).
All tests will be marked with `#[ignore]` to prevent them from running in CI by default.

## Test Configuration
- **Environment**: Testnet API (`https://api.testnet.lighter.xyz`)
- **Required Setup**:
  - Valid testnet private key (environment variable: `LIGHTER_TEST_PRIVATE_KEY`)
  - Valid API key (environment variable: `LIGHTER_TEST_API_KEY`)
  - Test market symbol: `BTC-USDC`
  - Test account with some balance

## Test Categories

### 1. Account Management Tests
#### 1.1 Account Information
- [ ] `test_get_account` - Retrieve account details
- [ ] `test_get_account_stats` - Get account statistics
- [ ] `test_get_balances` - Get all account balances
- [ ] `test_get_positions` - Get open positions

#### 1.2 Account Tier Management
- [ ] `test_can_switch_tier` - Check if tier switch is possible
- [ ] `test_change_account_tier` - Switch between Standard/Premium tiers
- [ ] `test_tier_switch_with_open_positions` - Verify tier switch fails with positions

### 2. Order Management Tests
#### 2.1 Order Lifecycle
- [ ] `test_create_limit_order` - Create a limit buy/sell order
- [ ] `test_create_market_order` - Create a market order
- [ ] `test_create_stop_loss_order` - Create stop loss order
- [ ] `test_create_take_profit_order` - Create take profit order
- [ ] `test_get_order_by_id` - Retrieve specific order
- [ ] `test_cancel_order` - Cancel a specific order
- [ ] `test_cancel_all_orders` - Cancel all orders

#### 2.2 Order Query
- [ ] `test_get_orders_no_filter` - Get all orders
- [ ] `test_get_orders_with_symbol_filter` - Get orders for specific symbol
- [ ] `test_get_orders_with_status_filter` - Get orders by status
- [ ] `test_get_orders_with_pagination` - Test pagination
- [ ] `test_get_trades` - Get trade history

#### 2.3 Order Edge Cases
- [ ] `test_create_order_insufficient_balance` - Verify proper error handling
- [ ] `test_cancel_non_existent_order` - Verify error on invalid order ID
- [ ] `test_create_order_invalid_price` - Test validation
- [ ] `test_create_post_only_order` - Test post-only flag
- [ ] `test_create_reduce_only_order` - Test reduce-only flag

### 3. Market Data Tests
#### 3.1 Candlestick Data
- [ ] `test_get_candlesticks_1m` - Get 1-minute candles
- [ ] `test_get_candlesticks_5m` - Get 5-minute candles
- [ ] `test_get_candlesticks_1h` - Get hourly candles
- [ ] `test_get_candlesticks_1d` - Get daily candles
- [ ] `test_get_candlesticks_with_limit` - Test limit parameter

#### 3.2 Market Statistics
- [ ] `test_get_market_stats` - Get stats for single market
- [ ] `test_get_all_market_stats` - Get stats for all markets
- [ ] `test_get_ticker` - Get ticker for single market
- [ ] `test_get_all_tickers` - Get all tickers

#### 3.3 Order Book
- [ ] `test_get_order_book_default` - Get order book with default depth
- [ ] `test_get_order_book_custom_depth` - Get order book with custom depth
- [ ] `test_order_book_spread_calculation` - Verify bid/ask spread

### 4. Transaction/Block Tests
#### 4.1 Transaction Queries
- [ ] `test_get_transaction_by_hash` - Get specific transaction
- [ ] `test_get_transactions_history` - Get transaction history
- [ ] `test_get_transactions_with_filter` - Filter by type

#### 4.2 Block Information
- [ ] `test_get_latest_block` - Get current block
- [ ] `test_get_block_by_number` - Get specific block
- [ ] `test_wait_for_confirmation` - Test confirmation waiting

### 5. WebSocket Tests
#### 5.1 Connection Management
- [ ] `test_ws_connect_disconnect` - Basic connection lifecycle
- [ ] `test_ws_reconnect` - Test reconnection logic

#### 5.2 Subscriptions
- [ ] `test_ws_subscribe_ticker` - Subscribe to ticker updates
- [ ] `test_ws_subscribe_orderbook` - Subscribe to order book updates
- [ ] `test_ws_subscribe_trades` - Subscribe to trade feed
- [ ] `test_ws_subscribe_account` - Subscribe to account updates
- [ ] `test_ws_multiple_subscriptions` - Handle multiple subscriptions
- [ ] `test_ws_unsubscribe` - Test unsubscription

#### 5.3 Real-time Updates
- [ ] `test_ws_order_fill_notification` - Receive order fill events
- [ ] `test_ws_balance_update` - Receive balance updates

### 6. Signer Tests
#### 6.1 Ethereum Signer
- [ ] `test_ethereum_signer_from_private_key` - Create from private key
- [ ] `test_ethereum_signer_from_mnemonic` - Create from mnemonic
- [ ] `test_ethereum_signer_sign_message` - Sign arbitrary message
- [ ] `test_ethereum_signer_get_address` - Verify address derivation

#### 6.2 FFI Signer (if binary available)
- [ ] `test_ffi_signer_initialization` - Initialize FFI signer
- [ ] `test_ffi_signer_create_order` - Sign order with FFI
- [ ] `test_ffi_signer_cancel_order` - Sign cancellation with FFI
- [ ] `test_ffi_signer_switch_api_key` - Test API key switching

### 7. Error Handling Tests
#### 7.1 Network Errors
- [ ] `test_network_timeout` - Handle request timeouts
- [ ] `test_rate_limit_handling` - Handle rate limiting
- [ ] `test_invalid_endpoint` - Handle 404 errors
- [ ] `test_server_error` - Handle 5xx errors

#### 7.2 Authentication Errors
- [ ] `test_invalid_api_key` - Handle auth failures
- [ ] `test_expired_signature` - Handle expired signatures
- [ ] `test_invalid_signature` - Handle invalid signatures

### 8. Concurrent Operations Tests
- [ ] `test_concurrent_order_creation` - Create multiple orders simultaneously
- [ ] `test_concurrent_data_fetching` - Fetch multiple data types concurrently
- [ ] `test_concurrent_ws_and_rest` - Use WebSocket and REST simultaneously

## Test Implementation Strategy

1. **Setup Phase**:
   - Create test configuration from environment variables
   - Initialize test client with proper credentials
   - Ensure test account has sufficient balance

2. **Test Execution**:
   - Each test should be independent and idempotent
   - Clean up created orders/positions after tests
   - Use deterministic test data where possible

3. **Assertion Strategy**:
   - Verify HTTP status codes
   - Check response structure matches models
   - Validate business logic (e.g., order total = price × quantity)
   - Ensure proper error messages for failure cases

4. **Cleanup Phase**:
   - Cancel all open orders
   - Close WebSocket connections
   - Log any remaining positions

## Running the Tests

```bash
# Run all integration tests (requires valid credentials)
LIGHTER_TEST_PRIVATE_KEY=0x... LIGHTER_TEST_API_KEY=... cargo test --ignored

# Run specific test category
cargo test --ignored test_get_account

# Run with logging
RUST_LOG=debug cargo test --ignored -- --nocapture
```

## Success Criteria
- All tests pass against testnet
- Proper error handling for all failure scenarios
- No resource leaks (connections, memory)
- Tests are maintainable and well-documented
- Test coverage for all public API methods