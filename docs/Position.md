# Position

Open position information for an account.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**symbol** | **String** | Trading pair symbol (e.g., "BTC-USDC") | 
**side** | [**Side**](Side.md) | Position side (Buy = Long, Sell = Short) | 
**size** | **String** | Position size | 
**entry_price** | **String** | Average entry price | 
**mark_price** | **String** | Current mark price | 
**unrealized_pnl** | **String** | Unrealized profit/loss | 
**margin_type** | [**MarginType**](MarginType.md) | Cross or Isolated margin | 
**leverage** | **String** | Position leverage | 
**created_at** | **DateTime<Utc>** | Position open timestamp | 
**updated_at** | **DateTime<Utc>** | Last update timestamp | 

## Example

```rust
use lighter_rust::{Position, Side, MarginType};
use chrono::Utc;

let position = Position {
    symbol: "BTC-USDC".to_string(),
    side: Side::Buy,  // Long position
    size: "0.5".to_string(),
    entry_price: "45000.00".to_string(),
    mark_price: "46000.00".to_string(),
    unrealized_pnl: "500.00".to_string(),
    margin_type: MarginType::Cross,
    leverage: "10".to_string(),
    created_at: Utc::now(),
    updated_at: Utc::now(),
};

// Calculate ROI
let entry: f64 = position.entry_price.parse().unwrap();
let mark: f64 = position.mark_price.parse().unwrap();
let roi = ((mark - entry) / entry) * 100.0;
println!("Position ROI: {:.2}%", roi);
```

## Position Management

### Opening a Position
Positions are opened when:
- A market order is executed
- A limit order is filled
- A stop/take-profit order is triggered

### Closing a Position
Positions can be closed by:
- Placing an opposite side order
- Using reduce-only orders
- Liquidation (if margin insufficient)

### Position Sizing
```rust
// Calculate position value
let size: f64 = position.size.parse().unwrap();
let price: f64 = position.mark_price.parse().unwrap();
let position_value = size * price;

// Calculate required margin
let leverage: f64 = position.leverage.parse().unwrap();
let required_margin = position_value / leverage;
```

## Profit & Loss Calculation

### Unrealized PnL
```rust
// For long positions
let pnl = (mark_price - entry_price) * size;

// For short positions  
let pnl = (entry_price - mark_price) * size;
```

### Realized PnL
Realized when position is partially or fully closed:
```rust
let realized_pnl = (exit_price - entry_price) * closed_size;
```

## Margin Types

### Cross Margin
- Shares margin across all positions
- Lower liquidation risk
- More capital efficient
- Default for most traders

### Isolated Margin
- Dedicated margin per position
- Limited loss to position margin
- Better risk management
- Recommended for high-risk trades

## Leverage

Leverage multiplies both profits and losses:
- **1x**: No leverage (spot-like)
- **5x**: 5 times capital efficiency
- **10x**: High leverage (higher risk)
- **20x+**: Very high risk

## Liquidation

Position is liquidated when:
```rust
let maintenance_margin_ratio = 0.005; // 0.5%
let liquidation_price = entry_price * (1 - 1/leverage + maintenance_margin_ratio);
```

## Risk Management

### Stop Loss
```rust
// Set stop loss 2% below entry for long
let stop_loss = entry_price * 0.98;
```

### Take Profit
```rust
// Set take profit 5% above entry for long
let take_profit = entry_price * 1.05;
```

### Position Sizing
```rust
// Risk only 1% of account per trade
let account_balance = 10000.0;
let risk_per_trade = account_balance * 0.01;
let stop_distance = entry_price * 0.02; // 2% stop
let position_size = risk_per_trade / stop_distance;
```

## Related Methods

- [`get_positions()`](AccountApi.md#get_positions) - Get all open positions
- [`create_order()`](OrderApi.md#create_order) - Open or modify positions
- [`get_account()`](AccountApi.md#get_account) - Get account with positions