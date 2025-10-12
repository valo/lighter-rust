use lighter_rust::api::CandlestickInterval;
use lighter_rust::{init_logging, Config, LighterClient, OrderType, Side};
use std::collections::VecDeque;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    // Configuration
    let config = Config::new().with_api_key("your-api-key").with_timeout(30);

    let client = LighterClient::new(config, "your-private-key")?;

    let symbol = "BTC-USDC";
    let trade_quantity = "0.001";
    let sma_period = 10;

    println!("Starting simple moving average trading bot for {}", symbol);
    println!(
        "SMA Period: {}, Trade Quantity: {}",
        sma_period, trade_quantity
    );

    let mut price_history: VecDeque<f64> = VecDeque::new();
    let mut position_open = false;
    let mut current_order_id: Option<String> = None;

    loop {
        // Get current market price
        let current_price = match get_current_price(&client, symbol).await {
            Ok(price) => price,
            Err(e) => {
                eprintln!("Failed to get current price: {}", e);
                sleep(Duration::from_secs(10)).await;
                continue;
            }
        };

        println!("Current price for {}: ${:.2}", symbol, current_price);

        // Add to price history
        price_history.push_back(current_price);

        // Keep only the last SMA period prices
        if price_history.len() > sma_period {
            price_history.pop_front();
        }

        // Calculate SMA if we have enough data points
        if price_history.len() == sma_period {
            let sma = price_history.iter().sum::<f64>() / sma_period as f64;
            println!("SMA({}): ${:.2}", sma_period, sma);

            // Simple trading logic: buy when price is above SMA, sell when below
            if current_price > sma && !position_open {
                println!("Signal: BUY (price above SMA)");

                match execute_buy_order(&client, symbol, trade_quantity).await {
                    Ok(order_id) => {
                        position_open = true;
                        current_order_id = Some(order_id.clone());
                        println!("Buy order placed: {}", order_id);
                    }
                    Err(e) => {
                        eprintln!("Failed to place buy order: {}", e);
                    }
                }
            } else if current_price < sma && position_open {
                println!("Signal: SELL (price below SMA)");

                match execute_sell_order(&client, symbol, trade_quantity).await {
                    Ok(order_id) => {
                        position_open = false;
                        current_order_id = Some(order_id.clone());
                        println!("Sell order placed: {}", order_id);
                    }
                    Err(e) => {
                        eprintln!("Failed to place sell order: {}", e);
                    }
                }
            } else {
                println!("No signal - holding current position");
            }
        } else {
            println!(
                "Collecting price data... ({}/{})",
                price_history.len(),
                sma_period
            );
        }

        // Check order status if we have an active order
        if let Some(ref order_id) = current_order_id {
            match client.orders().get_order(order_id).await {
                Ok(order) => {
                    println!("Order {} status: {:?}", order_id, order.status);

                    use lighter_rust::OrderStatus;
                    match order.status {
                        OrderStatus::Filled => {
                            println!(
                                "Order filled! Average price: {}",
                                order
                                    .average_fill_price
                                    .unwrap_or_else(|| "N/A".to_string())
                            );
                            current_order_id = None;
                        }
                        OrderStatus::Cancelled | OrderStatus::Rejected => {
                            println!("Order ended with status: {:?}", order.status);
                            current_order_id = None;
                            position_open = false; // Reset position on failed orders
                        }
                        _ => {
                            // Order still active
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to check order status: {}", e);
                }
            }
        }

        // Print account summary every 5 iterations
        if price_history.len() % 5 == 0 {
            print_account_summary(&client).await;
        }

        // Wait before next iteration
        sleep(Duration::from_secs(30)).await;
    }
}

async fn get_current_price(
    client: &LighterClient,
    symbol: &str,
) -> Result<f64, Box<dyn std::error::Error>> {
    let ticker = client.market_data().get_ticker(symbol).await?;
    Ok(ticker.price.parse()?)
}

async fn execute_buy_order(
    client: &LighterClient,
    symbol: &str,
    quantity: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Get current market price and place a limit order slightly above it
    let current_price = get_current_price(client, symbol).await?;
    let buy_price = current_price * 1.001; // 0.1% above market price

    let order = client
        .orders()
        .create_order(
            symbol,
            Side::Buy,
            OrderType::Limit,
            quantity,
            Some(&buy_price.to_string()),
            None,
            None,
            None,
            Some(true), // post_only
            None,
        )
        .await?;

    Ok(order.id)
}

async fn execute_sell_order(
    client: &LighterClient,
    symbol: &str,
    quantity: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Get current market price and place a limit order slightly below it
    let current_price = get_current_price(client, symbol).await?;
    let sell_price = current_price * 0.999; // 0.1% below market price

    let order = client
        .orders()
        .create_order(
            symbol,
            Side::Sell,
            OrderType::Limit,
            quantity,
            Some(&sell_price.to_string()),
            None,
            None,
            None,
            Some(true), // post_only
            None,
        )
        .await?;

    Ok(order.id)
}

async fn print_account_summary(client: &LighterClient) {
    println!("\n=== Account Summary ===");

    match client.account().get_account().await {
        Ok(account) => {
            println!("Account Tier: {:?}", account.tier);

            for balance in &account.balances {
                if balance.total.parse::<f64>().unwrap_or(0.0) > 0.0 {
                    println!(
                        "Balance {}: {} (Available: {})",
                        balance.asset, balance.total, balance.available
                    );
                }
            }

            if !account.positions.is_empty() {
                println!("Open Positions:");
                for position in &account.positions {
                    println!(
                        "  {}: {} {} @ {} (PnL: {})",
                        position.symbol,
                        match position.side {
                            Side::Buy => "LONG",
                            Side::Sell => "SHORT",
                        },
                        position.size,
                        position.entry_price,
                        position.unrealized_pnl
                    );
                }
            } else {
                println!("No open positions");
            }
        }
        Err(e) => {
            eprintln!("Failed to get account info: {}", e);
        }
    }
    println!("========================\n");
}
