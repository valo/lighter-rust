# Order

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique order identifier | 
**client_order_id** | **Option<String>** | Client-provided order identifier | [optional]
**symbol** | **String** | Trading pair symbol (e.g., "BTC-USDC") | 
**side** | [**Side**](Side.md) | Order side (Buy or Sell) | 
**order_type** | [**OrderType**](OrderType.md) | Order type | 
**status** | [**OrderStatus**](OrderStatus.md) | Current order status | 
**quantity** | **String** | Total order quantity | 
**price** | **Option<String>** | Limit price | [optional]
**stop_price** | **Option<String>** | Stop trigger price | [optional]
**filled_quantity** | **String** | Quantity already filled | 
**remaining_quantity** | **String** | Quantity remaining to fill | 
**average_fill_price** | **Option<String>** | Average execution price | [optional]
**fee** | **Option<String>** | Trading fee amount | [optional]
**time_in_force** | [**TimeInForce**](TimeInForce.md) | Time in force setting | 
**created_at** | **DateTime<Utc>** | Order creation timestamp | 
**updated_at** | **DateTime<Utc>** | Last update timestamp | 
**expires_at** | **Option<DateTime<Utc>>** | Order expiration timestamp | [optional]

## Example

```rust
use lighter_rust::{Order, Side, OrderType, OrderStatus, TimeInForce};
use chrono::Utc;

// Example of a limit buy order
let order = Order {
    id: "order_789".to_string(),
    client_order_id: Some("my_order_001".to_string()),
    symbol: "BTC-USDC".to_string(),
    side: Side::Buy,
    order_type: OrderType::Limit,
    status: OrderStatus::Open,
    quantity: "0.5".to_string(),
    price: Some("45000.00".to_string()),
    stop_price: None,
    filled_quantity: "0.1".to_string(),
    remaining_quantity: "0.4".to_string(),
    average_fill_price: Some("44999.50".to_string()),
    fee: Some("0.90".to_string()),
    time_in_force: TimeInForce::Gtc,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    expires_at: None,
};
```

## Order Types

### Market Order
- Executes immediately at best available price
- No price parameter required
- Cannot be post-only

### Limit Order
- Executes at specified price or better
- Requires price parameter
- Can be post-only for maker fees

### Stop Loss Order
- Triggers market order when stop price is reached
- Requires stop_price parameter
- Used for risk management

### Take Profit Order
- Triggers market order when profit target is reached
- Requires stop_price parameter
- Used for profit taking

## Order Status

- **Pending**: Order received but not yet processed
- **Open**: Active order in the order book
- **PartiallyFilled**: Some quantity executed
- **Filled**: Completely executed
- **Cancelled**: Cancelled by user or system
- **Rejected**: Rejected due to validation failure

## Time in Force

- **GTC** (Good Till Cancelled): Remains active until filled or cancelled
- **IOC** (Immediate Or Cancel): Fill immediately or cancel unfilled portion
- **FOK** (Fill Or Kill): Fill completely or cancel entirely
- **Day**: Valid for current trading day only

## Related Methods

- [`create_order()`](OrderApi.md#create_order) - Place a new order
- [`cancel_order()`](OrderApi.md#cancel_order) - Cancel an existing order
- [`get_order()`](OrderApi.md#get_order) - Get order details
- [`get_orders()`](OrderApi.md#get_orders) - List orders with filters