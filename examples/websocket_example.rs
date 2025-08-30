use lighter_rust::{init_logging, Config, WebSocketClient};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    // Create WebSocket client
    let config = Config::new();
    let mut ws_client = WebSocketClient::new(config);

    println!("Connecting to WebSocket...");
    ws_client.connect().await?;
    println!("Connected!");

    // Subscribe to order book updates for BTC-USDC
    let subscription_id = ws_client
        .subscribe(
            "orderbook",
            Some(json!({
                "symbol": "BTC-USDC",
                "depth": 10
            })),
        )
        .await?;

    println!(
        "Subscribed to order book updates with ID: {}",
        subscription_id
    );

    // Subscribe to trade updates
    let trade_subscription = ws_client
        .subscribe(
            "trades",
            Some(json!({
                "symbol": "BTC-USDC"
            })),
        )
        .await?;

    println!(
        "Subscribed to trade updates with ID: {}",
        trade_subscription
    );

    // Listen for messages for 30 seconds
    let mut message_count = 0;
    let start_time = std::time::Instant::now();

    while start_time.elapsed().as_secs() < 30 {
        match ws_client.next_message().await? {
            Some(message) => {
                message_count += 1;
                println!(
                    "Message #{}: {}",
                    message_count,
                    serde_json::to_string_pretty(&message)?
                );

                // Only show first 10 messages to avoid spam
                if message_count >= 10 {
                    println!("... (limiting output to first 10 messages)");
                    break;
                }
            }
            None => {
                println!("Connection closed");
                break;
            }
        }
    }

    // Unsubscribe from updates
    println!("Unsubscribing...");
    ws_client.unsubscribe(&subscription_id).await?;
    ws_client.unsubscribe(&trade_subscription).await?;

    // Close connection
    ws_client.close().await?;
    println!("Connection closed gracefully");

    Ok(())
}
