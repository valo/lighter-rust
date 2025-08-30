use lighter_rust::{AccountTier, Config, LighterClient, OrderType, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::init();

    // Create configuration
    let config = Config::new().with_api_key("your-api-key").with_timeout(60);

    // Initialize client with private key
    let client = LighterClient::new(config, "your-private-key")?;

    println!("=== Account Information ===");

    // Get account info
    match client.account().get_account().await {
        Ok(account) => {
            println!("Account ID: {}", account.id);
            println!("Address: {}", account.address);
            println!("Tier: {:?}", account.tier);
            println!("Balances: {}", account.balances.len());
            println!("Positions: {}", account.positions.len());
        }
        Err(e) => {
            println!("Failed to get account: {}", e);
        }
    }

    // Get account statistics
    match client.account().get_account_stats().await {
        Ok(stats) => {
            println!("\n=== Account Statistics ===");
            println!("Total Volume: {}", stats.total_volume);
            println!("Total Trades: {}", stats.total_trades);
            println!("Win Rate: {}%", stats.win_rate);
        }
        Err(e) => {
            println!("Failed to get account stats: {}", e);
        }
    }

    println!("\n=== Market Data ===");

    // Get market stats for a symbol
    let symbol = "BTC-USDC";
    match client.market_data().get_market_stats(symbol).await {
        Ok(stats) => {
            println!("Symbol: {}", stats.symbol);
            println!("Last Price: {}", stats.last_price);
            println!("24h Change: {}%", stats.price_change_percent);
            println!("24h Volume: {}", stats.volume);
        }
        Err(e) => {
            println!("Failed to get market stats for {}: {}", symbol, e);
        }
    }

    // Get order book
    match client.market_data().get_order_book(symbol, Some(10)).await {
        Ok(orderbook) => {
            println!("\n=== Order Book ===");
            println!(
                "Best Bid: {} @ {}",
                orderbook
                    .bids
                    .first()
                    .map(|b| &b.quantity)
                    .unwrap_or(&"N/A".to_string()),
                orderbook
                    .bids
                    .first()
                    .map(|b| &b.price)
                    .unwrap_or(&"N/A".to_string())
            );
            println!(
                "Best Ask: {} @ {}",
                orderbook
                    .asks
                    .first()
                    .map(|a| &a.quantity)
                    .unwrap_or(&"N/A".to_string()),
                orderbook
                    .asks
                    .first()
                    .map(|a| &a.price)
                    .unwrap_or(&"N/A".to_string())
            );
        }
        Err(e) => {
            println!("Failed to get order book: {}", e);
        }
    }

    println!("\n=== Trading Operations ===");

    // Example: Place a limit buy order (this will likely fail without proper setup)
    match client
        .orders()
        .create_order(
            symbol,
            Side::Buy,
            OrderType::Limit,
            "0.001",       // quantity
            Some("45000"), // price
            None,          // client_order_id
            None,          // time_in_force (defaults to GTC)
            Some(true),    // post_only
            None,          // reduce_only
        )
        .await
    {
        Ok(order) => {
            println!("Order created successfully:");
            println!("Order ID: {}", order.id);
            println!("Status: {:?}", order.status);
        }
        Err(e) => {
            println!("Failed to create order: {}", e);
            println!("This is expected if you don't have proper API credentials and balance");
        }
    }

    // Get recent orders
    match client.orders().get_orders(None).await {
        Ok((orders, pagination)) => {
            println!("\n=== Recent Orders ===");
            println!("Found {} orders", orders.len());
            for order in orders.iter().take(5) {
                println!(
                    "Order {}: {} {} {} @ {} - {:?}",
                    order.id,
                    order.symbol,
                    match order.side {
                        Side::Buy => "BUY",
                        Side::Sell => "SELL",
                    },
                    order.quantity,
                    order.price.as_ref().unwrap_or(&"MARKET".to_string()),
                    order.status
                );
            }

            if let Some(pagination) = pagination {
                println!(
                    "Page {}/{}, Total: {}",
                    pagination.page,
                    (pagination.total + pagination.limit as u64 - 1) / pagination.limit as u64,
                    pagination.total
                );
            }
        }
        Err(e) => {
            println!("Failed to get orders: {}", e);
        }
    }

    Ok(())
}
