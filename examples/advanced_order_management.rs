use chrono::Utc;
use lighter_rust::{
    init_logging_with_filter, Config, LighterClient, OrderStatus, OrderType, Side,
    TimeInForce,
};
use lighter_rust::models::OrderFilter;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

/// Advanced order management system with:
/// - Order tracking and monitoring
/// - Stop loss and take profit management
/// - Grid trading strategy
/// - Order modification and replacement
/// - Risk management

#[derive(Debug)]
struct OrderManager {
    client: LighterClient,
    active_orders: HashMap<String, OrderInfo>,
    position_limits: HashMap<String, PositionLimit>,
    max_orders_per_symbol: usize,
}

#[derive(Debug, Clone)]
struct OrderInfo {
    order_id: String,
    symbol: String,
    side: Side,
    quantity: f64,
    price: Option<f64>,
    order_type: OrderType,
    created_at: chrono::DateTime<Utc>,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
}

#[derive(Debug, Clone)]
struct PositionLimit {
    symbol: String,
    max_position: f64,
    current_position: f64,
    max_loss: f64,
}

impl OrderManager {
    fn new(client: LighterClient) -> Self {
        Self {
            client,
            active_orders: HashMap::new(),
            position_limits: HashMap::new(),
            max_orders_per_symbol: 10,
        }
    }

    /// Place an order with automatic stop loss and take profit
    async fn place_order_with_sl_tp(
        &mut self,
        symbol: &str,
        side: Side,
        quantity: f64,
        price: f64,
        stop_loss_pct: f64,
        take_profit_pct: f64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Check position limits
        if !self.check_position_limit(symbol, quantity, side.clone()) {
            return Err("Position limit exceeded".into());
        }

        // Check max orders per symbol
        let symbol_orders = self
            .active_orders
            .values()
            .filter(|o| o.symbol == symbol)
            .count();

        if symbol_orders >= self.max_orders_per_symbol {
            return Err(format!(
                "Max orders ({}) reached for {}",
                self.max_orders_per_symbol, symbol
            )
            .into());
        }

        // Place main order
        println!(
            "Placing {} order for {} {} @ {}",
            match side {
                Side::Buy => "BUY",
                Side::Sell => "SELL",
            },
            quantity,
            symbol,
            price
        );

        let main_order = self
            .client
            .orders()
            .create_order(
                symbol,
                side.clone(),
                OrderType::Limit,
                &quantity.to_string(),
                Some(&price.to_string()),
                None,
                Some(TimeInForce::Gtc),
                Some(true), // post_only
                None,
            )
            .await?;

        let order_id = main_order.id.clone();

        // Calculate stop loss and take profit prices
        let (sl_price, tp_price) = match side {
            Side::Buy => {
                let sl = price * (1.0 - stop_loss_pct / 100.0);
                let tp = price * (1.0 + take_profit_pct / 100.0);
                (sl, tp)
            }
            Side::Sell => {
                let sl = price * (1.0 + stop_loss_pct / 100.0);
                let tp = price * (1.0 - take_profit_pct / 100.0);
                (sl, tp)
            }
        };

        // Store order info
        let order_info = OrderInfo {
            order_id: order_id.clone(),
            symbol: symbol.to_string(),
            side,
            quantity,
            price: Some(price),
            order_type: OrderType::Limit,
            created_at: Utc::now(),
            stop_loss: Some(sl_price),
            take_profit: Some(tp_price),
        };

        self.active_orders.insert(order_id.clone(), order_info);

        println!("Order {} placed successfully", order_id);
        println!("  Stop Loss: {:.2}", sl_price);
        println!("  Take Profit: {:.2}", tp_price);

        Ok(order_id)
    }

    /// Implement a grid trading strategy
    async fn setup_grid_orders(
        &mut self,
        symbol: &str,
        base_price: f64,
        grid_size: f64,
        grid_levels: usize,
        quantity_per_level: f64,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut order_ids = Vec::new();

        println!("Setting up grid orders for {}", symbol);
        println!(
            "Base price: {}, Grid size: {}%, Levels: {}",
            base_price, grid_size, grid_levels
        );

        for i in 1..=grid_levels {
            // Buy orders below current price
            let buy_price = base_price * (1.0 - (grid_size * i as f64) / 100.0);
            match self
                .client
                .orders()
                .create_order(
                    symbol,
                    Side::Buy,
                    OrderType::Limit,
                    &quantity_per_level.to_string(),
                    Some(&buy_price.to_string()),
                    None,
                    Some(TimeInForce::Gtc),
                    Some(true),
                    None,
                )
                .await
            {
                Ok(order) => {
                    println!(
                        "  Grid BUY #{}: {} @ {:.2}",
                        i, quantity_per_level, buy_price
                    );
                    order_ids.push(order.id.clone());

                    self.active_orders.insert(
                        order.id.clone(),
                        OrderInfo {
                            order_id: order.id,
                            symbol: symbol.to_string(),
                            side: Side::Buy,
                            quantity: quantity_per_level,
                            price: Some(buy_price),
                            order_type: OrderType::Limit,
                            created_at: Utc::now(),
                            stop_loss: None,
                            take_profit: None,
                        },
                    );
                }
                Err(e) => {
                    eprintln!("Failed to place grid buy order: {}", e);
                }
            }

            // Sell orders above current price
            let sell_price = base_price * (1.0 + (grid_size * i as f64) / 100.0);
            match self
                .client
                .orders()
                .create_order(
                    symbol,
                    Side::Sell,
                    OrderType::Limit,
                    &quantity_per_level.to_string(),
                    Some(&sell_price.to_string()),
                    None,
                    Some(TimeInForce::Gtc),
                    Some(true),
                    None,
                )
                .await
            {
                Ok(order) => {
                    println!(
                        "  Grid SELL #{}: {} @ {:.2}",
                        i, quantity_per_level, sell_price
                    );
                    order_ids.push(order.id.clone());

                    self.active_orders.insert(
                        order.id.clone(),
                        OrderInfo {
                            order_id: order.id,
                            symbol: symbol.to_string(),
                            side: Side::Sell,
                            quantity: quantity_per_level,
                            price: Some(sell_price),
                            order_type: OrderType::Limit,
                            created_at: Utc::now(),
                            stop_loss: None,
                            take_profit: None,
                        },
                    );
                }
                Err(e) => {
                    eprintln!("Failed to place grid sell order: {}", e);
                }
            }

            // Small delay to avoid rate limits
            sleep(Duration::from_millis(100)).await;
        }

        println!("Grid setup complete: {} orders placed", order_ids.len());
        Ok(order_ids)
    }

    /// Monitor and manage active orders
    async fn monitor_orders(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Monitoring {} active orders...", self.active_orders.len());

        let order_ids: Vec<String> = self.active_orders.keys().cloned().collect();

        for order_id in order_ids {
            match self.client.orders().get_order(&order_id).await {
                Ok(order) => {
                    println!("Order {}: {:?}", order_id, order.status);

                    match order.status {
                        OrderStatus::Filled => {
                            if let Some(order_info) = self.active_orders.get(&order_id) {
                                println!("  Order filled! Processing SL/TP...");
                                self.handle_filled_order(order_info.clone()).await?;
                            }
                            self.active_orders.remove(&order_id);
                        }
                        OrderStatus::Cancelled | OrderStatus::Rejected => {
                            println!("  Order ended: {:?}", order.status);
                            self.active_orders.remove(&order_id);
                        }
                        OrderStatus::PartiallyFilled => {
                            println!(
                                "  Partially filled: {}/{}",
                                order.filled_quantity, order.quantity
                            );
                        }
                        _ => {
                            // Order still active
                            if let Some(order_info) = self.active_orders.get(&order_id) {
                                // Check if order is too old (e.g., 1 hour)
                                let age = Utc::now() - order_info.created_at;
                                if age.num_hours() > 1 {
                                    println!(
                                        "  Order is {} hours old, considering cancellation",
                                        age.num_hours()
                                    );
                                    // Optionally cancel old orders
                                    // self.client.orders().cancel_order(Some(&order_id), None, None).await?;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to get order {}: {}", order_id, e);
                }
            }

            sleep(Duration::from_millis(50)).await; // Rate limit protection
        }

        Ok(())
    }

    /// Handle a filled order by placing stop loss and take profit orders
    async fn handle_filled_order(
        &mut self,
        order_info: OrderInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Place stop loss order if configured
        if let Some(sl_price) = order_info.stop_loss {
            let sl_side = match order_info.side {
                Side::Buy => Side::Sell, // Sell to stop loss on a buy position
                Side::Sell => Side::Buy, // Buy to stop loss on a sell position
            };

            println!("  Placing stop loss at {:.2}", sl_price);
            match self
                .client
                .orders()
                .create_order(
                    &order_info.symbol,
                    sl_side,
                    OrderType::StopLoss,
                    &order_info.quantity.to_string(),
                    None,
                    None,
                    Some(TimeInForce::Gtc),
                    None,
                    Some(true), // reduce_only
                )
                .await
            {
                Ok(sl_order) => {
                    println!("  Stop loss order placed: {}", sl_order.id);
                }
                Err(e) => {
                    eprintln!("  Failed to place stop loss: {}", e);
                }
            }
        }

        // Place take profit order if configured
        if let Some(tp_price) = order_info.take_profit {
            let tp_side = match order_info.side {
                Side::Buy => Side::Sell, // Sell to take profit on a buy position
                Side::Sell => Side::Buy, // Buy to take profit on a sell position
            };

            println!("  Placing take profit at {:.2}", tp_price);
            match self
                .client
                .orders()
                .create_order(
                    &order_info.symbol,
                    tp_side,
                    OrderType::Limit,
                    &order_info.quantity.to_string(),
                    Some(&tp_price.to_string()),
                    None,
                    Some(TimeInForce::Gtc),
                    None,
                    Some(true), // reduce_only
                )
                .await
            {
                Ok(tp_order) => {
                    println!("  Take profit order placed: {}", tp_order.id);
                }
                Err(e) => {
                    eprintln!("  Failed to place take profit: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Cancel all orders for a symbol
    async fn cancel_all_symbol_orders(
        &mut self,
        symbol: &str,
    ) -> Result<u32, Box<dyn std::error::Error>> {
        println!("Cancelling all orders for {}", symbol);

        let count = self.client.orders().cancel_all_orders(Some(symbol)).await?;

        // Remove from local tracking
        self.active_orders.retain(|_, order| order.symbol != symbol);

        println!("Cancelled {} orders", count);
        Ok(count)
    }

    /// Check if a new order would exceed position limits
    fn check_position_limit(&self, symbol: &str, quantity: f64, side: Side) -> bool {
        if let Some(limit) = self.position_limits.get(symbol) {
            let position_change = match side {
                Side::Buy => quantity,
                Side::Sell => -quantity,
            };

            let new_position = limit.current_position + position_change;

            if new_position.abs() > limit.max_position {
                println!(
                    "Position limit exceeded for {}: {} > {}",
                    symbol,
                    new_position.abs(),
                    limit.max_position
                );
                return false;
            }
        }
        true
    }

    /// Set position limit for a symbol
    fn set_position_limit(&mut self, symbol: &str, max_position: f64, max_loss: f64) {
        self.position_limits.insert(
            symbol.to_string(),
            PositionLimit {
                symbol: symbol.to_string(),
                max_position,
                current_position: 0.0,
                max_loss,
            },
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging_with_filter("lighter_rust=debug");

    // Create configuration
    let config = Config::new().with_api_key("your-api-key").with_timeout(30);

    // Initialize client
    let client = LighterClient::new(config, "your-private-key")?;

    // Create order manager
    let mut manager = OrderManager::new(client);

    // Set position limits
    manager.set_position_limit("BTC-USDC", 1.0, 1000.0); // Max 1 BTC position, $1000 max loss
    manager.set_position_limit("ETH-USDC", 10.0, 500.0); // Max 10 ETH position, $500 max loss

    println!("=== Advanced Order Management System ===\n");

    // Example 1: Place order with stop loss and take profit
    println!("1. Placing order with SL/TP...");
    match manager
        .place_order_with_sl_tp(
            "BTC-USDC",
            Side::Buy,
            0.01,    // quantity
            45000.0, // price
            2.0,     // 2% stop loss
            5.0,     // 5% take profit
        )
        .await
    {
        Ok(order_id) => println!("Order placed: {}\n", order_id),
        Err(e) => eprintln!("Failed to place order: {}\n", e),
    }

    sleep(Duration::from_secs(2)).await;

    // Example 2: Setup grid trading
    println!("2. Setting up grid trading...");
    match manager
        .setup_grid_orders(
            "ETH-USDC", 3000.0, // base price
            1.0,    // 1% grid size
            5,      // 5 levels
            0.1,    // 0.1 ETH per level
        )
        .await
    {
        Ok(order_ids) => println!("Grid orders placed: {} orders\n", order_ids.len()),
        Err(e) => eprintln!("Failed to setup grid: {}\n", e),
    }

    sleep(Duration::from_secs(2)).await;

    // Example 3: Monitor orders
    println!("3. Monitoring orders...");
    for _ in 0..3 {
        manager.monitor_orders().await?;
        sleep(Duration::from_secs(5)).await;
    }

    // Example 4: Query and manage orders
    println!("\n4. Querying open orders...");
    let filter = OrderFilter {
        symbol: Some("BTC-USDC".to_string()),
        status: Some(OrderStatus::Open),
        side: None,
        order_type: None,
        start_time: None,
        end_time: None,
        page: Some(1),
        limit: Some(10),
    };

    match manager.client.orders().get_orders(Some(filter)).await {
        Ok((orders, _pagination)) => {
            println!("Found {} open BTC-USDC orders", orders.len());
            for order in orders.iter().take(5) {
                println!(
                    "  {} {} {} @ {} - {:?}",
                    order.id,
                    match order.side {
                        Side::Buy => "BUY",
                        Side::Sell => "SELL",
                    },
                    order.quantity,
                    order.price.as_ref().unwrap_or(&"MARKET".to_string()),
                    order.status
                );
            }
        }
        Err(e) => eprintln!("Failed to query orders: {}", e),
    }

    // Example 5: Risk management - cancel all if needed
    println!("\n5. Risk Management Demo...");
    println!("Would cancel all ETH-USDC orders in production");
    // Uncomment to actually cancel:
    // manager.cancel_all_symbol_orders("ETH-USDC").await?;

    println!("\n=== Order Management Complete ===");
    println!("Active orders remaining: {}", manager.active_orders.len());

    Ok(())
}
