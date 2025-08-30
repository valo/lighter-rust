# WebSocketClient

Real-time WebSocket client for streaming market data and account updates.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**config** | [**Config**](Config.md) | WebSocket configuration | 
**stream** | **Option<WsStream>** | Active WebSocket connection | [optional]
**subscriptions** | **HashMap<String, String>** | Active subscription mappings | 

## Methods

### connect

```rust
pub async fn connect(&mut self) -> Result<()>
```

Establishes WebSocket connection to the server.

### subscribe

```rust
pub async fn subscribe(&mut self, channel: &str, params: Option<Value>) -> Result<String>
```

Subscribe to a specific data channel.

### unsubscribe

```rust
pub async fn unsubscribe(&mut self, subscription_id: &str) -> Result<()>
```

Unsubscribe from a channel.

### next_message

```rust
pub async fn next_message(&mut self) -> Result<Option<Value>>
```

Receive the next message from the WebSocket stream.

### close

```rust
pub async fn close(&mut self) -> Result<()>
```

Close the WebSocket connection gracefully.

## Example Usage

```rust
use lighter_rust::{Config, WebSocketClient};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_ws_url("wss://ws.lighter.xyz")?;
    
    let mut ws_client = WebSocketClient::new(config);
    
    // Connect to WebSocket
    ws_client.connect().await?;
    println!("Connected to WebSocket");
    
    // Subscribe to order book updates
    let orderbook_sub = ws_client.subscribe(
        "orderbook",
        Some(json!({
            "symbol": "BTC-USDC",
            "depth": 10
        }))
    ).await?;
    
    // Subscribe to trade feed
    let trades_sub = ws_client.subscribe(
        "trades",
        Some(json!({ "symbol": "BTC-USDC" }))
    ).await?;
    
    // Process messages
    loop {
        match ws_client.next_message().await? {
            Some(message) => {
                // Handle different message types
                if let Some(msg_type) = message.get("type") {
                    match msg_type.as_str() {
                        Some("orderbook") => handle_orderbook_update(&message),
                        Some("trade") => handle_trade(&message),
                        Some("error") => handle_error(&message),
                        _ => {}
                    }
                }
            }
            None => {
                println!("Connection closed");
                break;
            }
        }
    }
    
    // Clean up
    ws_client.unsubscribe(&orderbook_sub).await?;
    ws_client.unsubscribe(&trades_sub).await?;
    ws_client.close().await?;
    
    Ok(())
}

fn handle_orderbook_update(message: &serde_json::Value) {
    println!("Order book update: {:?}", message);
}

fn handle_trade(message: &serde_json::Value) {
    println!("New trade: {:?}", message);
}

fn handle_error(message: &serde_json::Value) {
    eprintln!("WebSocket error: {:?}", message);
}
```

## Subscription Channels

### Order Book
```rust
ws_client.subscribe("orderbook", Some(json!({
    "symbol": "BTC-USDC",
    "depth": 20  // Number of price levels
})))
```

### Trades
```rust
ws_client.subscribe("trades", Some(json!({
    "symbol": "BTC-USDC"
})))
```

### Ticker
```rust
ws_client.subscribe("ticker", Some(json!({
    "symbol": "BTC-USDC"
})))
```

### Account Updates
```rust
ws_client.subscribe("account", Some(json!({
    "account_id": "your_account_id"
})))
```

### Order Updates
```rust
ws_client.subscribe("orders", Some(json!({
    "account_id": "your_account_id"
})))
```

## Message Format

All WebSocket messages follow this general structure:

```json
{
    "id": "subscription_id",
    "type": "message_type",
    "channel": "channel_name",
    "data": {
        // Channel-specific data
    },
    "timestamp": "2024-01-01T00:00:00Z"
}
```

## Error Handling

```rust
match ws_client.next_message().await {
    Ok(Some(msg)) => process_message(msg),
    Ok(None) => {
        // Connection closed normally
        reconnect().await?;
    }
    Err(LighterError::WebSocket(e)) => {
        // Handle WebSocket errors
        eprintln!("WebSocket error: {}", e);
        reconnect().await?;
    }
    Err(e) => {
        // Handle other errors
        eprintln!("Error: {}", e);
    }
}
```

## Reconnection Strategy

```rust
async fn maintain_connection(mut ws_client: WebSocketClient) {
    let mut retry_count = 0;
    let max_retries = 5;
    
    loop {
        if !ws_client.is_connected() {
            if retry_count >= max_retries {
                eprintln!("Max reconnection attempts reached");
                break;
            }
            
            let delay = std::time::Duration::from_secs(2_u64.pow(retry_count));
            tokio::time::sleep(delay).await;
            
            match ws_client.connect().await {
                Ok(_) => {
                    println!("Reconnected successfully");
                    retry_count = 0;
                    // Re-subscribe to channels
                }
                Err(e) => {
                    eprintln!("Reconnection failed: {}", e);
                    retry_count += 1;
                }
            }
        }
        
        // Process messages...
    }
}
```

## Performance Considerations

- **Message Buffering**: The client buffers incoming messages internally
- **Backpressure**: Implement proper backpressure handling for high-frequency streams
- **Heartbeat**: Automatic ping/pong for connection health
- **Compression**: Supports message compression for bandwidth optimization

## Related

- [Config](Config.md) - WebSocket configuration
- [LighterClient](LighterClient.md) - Main client with WebSocket integration