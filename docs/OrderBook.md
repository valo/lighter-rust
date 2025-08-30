# OrderBook

Market depth information showing current buy and sell orders.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**bids** | **Vec<PriceLevel>** | Buy orders sorted by price (highest first) | 
**asks** | **Vec<PriceLevel>** | Sell orders sorted by price (lowest first) | 
**timestamp** | **DateTime<Utc>** | Order book snapshot timestamp | 

## PriceLevel

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**price** | **String** | Price level | 
**quantity** | **String** | Total quantity at this price | 

## Example

```rust
use lighter_rust::{OrderBook, PriceLevel};
use chrono::Utc;

let order_book = OrderBook {
    bids: vec![
        PriceLevel { price: "45000.00".to_string(), quantity: "2.5".to_string() },
        PriceLevel { price: "44999.00".to_string(), quantity: "5.0".to_string() },
        PriceLevel { price: "44998.00".to_string(), quantity: "3.2".to_string() },
    ],
    asks: vec![
        PriceLevel { price: "45001.00".to_string(), quantity: "1.8".to_string() },
        PriceLevel { price: "45002.00".to_string(), quantity: "4.5".to_string() },
        PriceLevel { price: "45003.00".to_string(), quantity: "2.1".to_string() },
    ],
    timestamp: Utc::now(),
};

// Calculate spread
let best_bid: f64 = order_book.bids[0].price.parse().unwrap();
let best_ask: f64 = order_book.asks[0].price.parse().unwrap();
let spread = best_ask - best_bid;
let spread_percentage = (spread / best_ask) * 100.0;

println!("Spread: ${:.2} ({:.3}%)", spread, spread_percentage);
```

## Order Book Analysis

### Market Depth
```rust
fn calculate_depth(levels: &[PriceLevel], max_levels: usize) -> f64 {
    levels.iter()
        .take(max_levels)
        .map(|level| {
            let price: f64 = level.price.parse().unwrap();
            let quantity: f64 = level.quantity.parse().unwrap();
            price * quantity
        })
        .sum()
}

let bid_depth = calculate_depth(&order_book.bids, 10);
let ask_depth = calculate_depth(&order_book.asks, 10);
println!("Bid depth (10 levels): ${:.2}", bid_depth);
println!("Ask depth (10 levels): ${:.2}", ask_depth);
```

### Order Book Imbalance
```rust
fn calculate_imbalance(order_book: &OrderBook, levels: usize) -> f64 {
    let bid_volume: f64 = order_book.bids.iter()
        .take(levels)
        .map(|l| l.quantity.parse::<f64>().unwrap())
        .sum();
    
    let ask_volume: f64 = order_book.asks.iter()
        .take(levels)
        .map(|l| l.quantity.parse::<f64>().unwrap())
        .sum();
    
    (bid_volume - ask_volume) / (bid_volume + ask_volume)
}

let imbalance = calculate_imbalance(&order_book, 5);
if imbalance > 0.2 {
    println!("Strong buying pressure");
} else if imbalance < -0.2 {
    println!("Strong selling pressure");
}
```

### Support and Resistance Levels
```rust
fn find_support_resistance(order_book: &OrderBook) -> (Vec<f64>, Vec<f64>) {
    // Support levels - large bid quantities
    let support: Vec<f64> = order_book.bids.iter()
        .filter(|level| {
            let qty: f64 = level.quantity.parse().unwrap();
            qty > 10.0  // Threshold for significant level
        })
        .map(|level| level.price.parse().unwrap())
        .collect();
    
    // Resistance levels - large ask quantities
    let resistance: Vec<f64> = order_book.asks.iter()
        .filter(|level| {
            let qty: f64 = level.quantity.parse().unwrap();
            qty > 10.0
        })
        .map(|level| level.price.parse().unwrap())
        .collect();
    
    (support, resistance)
}
```

## WebSocket Streaming

Subscribe to real-time order book updates:

```rust
use lighter_rust::WebSocketClient;
use serde_json::json;

let mut ws = WebSocketClient::new(config);
ws.connect().await?;

let sub_id = ws.subscribe("orderbook", Some(json!({
    "symbol": "BTC-USDC",
    "depth": 20,
    "update_speed": 100  // milliseconds
}))).await?;

while let Some(msg) = ws.next_message().await? {
    if let Some(data) = msg.get("data") {
        let order_book: OrderBook = serde_json::from_value(data.clone())?;
        process_order_book(&order_book);
    }
}
```

## Trading Strategies

### Market Making
```rust
fn calculate_quotes(order_book: &OrderBook, spread_bps: f64) -> (f64, f64) {
    let mid_price = (
        order_book.bids[0].price.parse::<f64>().unwrap() +
        order_book.asks[0].price.parse::<f64>().unwrap()
    ) / 2.0;
    
    let half_spread = mid_price * (spread_bps / 10000.0);
    let bid_price = mid_price - half_spread;
    let ask_price = mid_price + half_spread;
    
    (bid_price, ask_price)
}
```

### Arbitrage Detection
```rust
fn detect_arbitrage(book1: &OrderBook, book2: &OrderBook) -> Option<f64> {
    let best_bid1: f64 = book1.bids[0].price.parse().unwrap();
    let best_ask2: f64 = book2.asks[0].price.parse().unwrap();
    
    if best_bid1 > best_ask2 {
        Some(best_bid1 - best_ask2)
    } else {
        None
    }
}
```

## Performance Considerations

- **Depth Limit**: Request only needed depth levels
- **Update Frequency**: Balance between latency and bandwidth
- **Snapshot vs Delta**: Use delta updates for efficiency
- **Local Order Book**: Maintain local copy with updates

## Related Methods

- [`get_order_book()`](CandlestickApi.md#get_order_book) - Get order book snapshot
- [WebSocketClient](WebSocketClient.md) - Stream order book updates