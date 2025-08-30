# OrderApi

All URIs are relative to *https://api.lighter.xyz*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_order**](OrderApi.md#create_order) | **POST** /orders | Create a new order
[**cancel_order**](OrderApi.md#cancel_order) | **POST** /orders/cancel | Cancel an order
[**cancel_all_orders**](OrderApi.md#cancel_all_orders) | **POST** /orders/cancel-all | Cancel all orders
[**get_order**](OrderApi.md#get_order) | **GET** /orders/{order_id} | Get order by ID
[**get_orders**](OrderApi.md#get_orders) | **GET** /orders | Get orders with filters
[**get_trades**](OrderApi.md#get_trades) | **GET** /trades | Get trade history

## create_order

> Order create_order(symbol, side, order_type, quantity, price, client_order_id, time_in_force, post_only, reduce_only)

Create a new order

Places a new order on the exchange.

### Example

```rust
use lighter_rust::{LighterClient, Config, Side, OrderType, TimeInForce};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    // Place a limit buy order
    let order = client.orders().create_order(
        "BTC-USDC",
        Side::Buy,
        OrderType::Limit,
        "0.1",           // quantity
        Some("45000"),   // price
        None,            // client_order_id
        Some(TimeInForce::Gtc),
        Some(true),      // post_only
        None,            // reduce_only
    ).await?;
    
    println!("Order placed: {}", order.id);
    println!("Status: {:?}", order.status);
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**symbol** | **String** | Trading pair symbol | [required] |
**side** | [**Side**](Side.md) | Buy or Sell | [required] |
**order_type** | [**OrderType**](OrderType.md) | Order type | [required] |
**quantity** | **String** | Order quantity | [required] |
**price** | **Option<String>** | Limit price (required for limit orders) | [optional] |
**client_order_id** | **Option<String>** | Client-provided order ID | [optional] |
**time_in_force** | **Option<TimeInForce>** | Time in force | [optional] [default to GTC]
**post_only** | **Option<bool>** | Post-only order | [optional] [default to false]
**reduce_only** | **Option<bool>** | Reduce-only order | [optional] [default to false]

### Return type

[**Order**](Order.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [SignatureAuth](../README.md#SignatureAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

## cancel_order

> cancel_order(order_id, client_order_id, symbol)

Cancel an order

Cancels a specific order by ID or client order ID.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    // Cancel by order ID
    client.orders().cancel_order(
        Some("order123"),
        None,
        None,
    ).await?;
    
    println!("Order cancelled successfully");
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**order_id** | **Option<String>** | Order ID | [optional] |
**client_order_id** | **Option<String>** | Client order ID | [optional] |
**symbol** | **Option<String>** | Trading pair symbol | [optional] |

### Return type

(empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [SignatureAuth](../README.md#SignatureAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

## cancel_all_orders

> u32 cancel_all_orders(symbol)

Cancel all orders

Cancels all open orders, optionally filtered by symbol.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    // Cancel all BTC-USDC orders
    let cancelled_count = client.orders()
        .cancel_all_orders(Some("BTC-USDC"))
        .await?;
    
    println!("Cancelled {} orders", cancelled_count);
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**symbol** | **Option<String>** | Filter by trading pair | [optional] |

### Return type

**u32**

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [SignatureAuth](../README.md#SignatureAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

## get_order

> Order get_order(order_id)

Get order by ID

Retrieves details of a specific order.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    let order = client.orders().get_order("order123").await?;
    
    println!("Order: {} - {:?}", order.id, order.status);
    println!("Filled: {}/{}", order.filled_quantity, order.quantity);
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**order_id** | **String** | Order ID | [required] |

### Return type

[**Order**](Order.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_orders

> (Vec<Order>, Option<Pagination>) get_orders(filter)

Get orders with filters

Retrieves orders with optional filtering.

### Example

```rust
use lighter_rust::{LighterClient, Config, OrderFilter, OrderStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    let filter = OrderFilter {
        symbol: Some("BTC-USDC".to_string()),
        status: Some(OrderStatus::Open),
        side: None,
        order_type: None,
        start_time: None,
        end_time: None,
        page: Some(1),
        limit: Some(50),
    };
    
    let (orders, pagination) = client.orders()
        .get_orders(Some(filter))
        .await?;
    
    for order in orders {
        println!("{}: {} {} @ {}", 
            order.id,
            order.symbol,
            order.quantity,
            order.price.unwrap_or("MARKET".to_string())
        );
    }
    
    if let Some(p) = pagination {
        println!("Page {}/{}", p.page, p.total / p.limit as u64 + 1);
    }
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**filter** | **Option<OrderFilter>** | Filter parameters | [optional] |

### Return type

(**Vec<Order>**, **Option<Pagination>**)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_trades

> Vec<Trade> get_trades(symbol)

Get trade history

Returns executed trades for the account.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    // Get all BTC-USDC trades
    let trades = client.orders()
        .get_trades(Some("BTC-USDC"))
        .await?;
    
    for trade in trades {
        println!("Trade {}: {} {} @ {} - Fee: {} {}", 
            trade.id,
            trade.quantity,
            trade.symbol,
            trade.price,
            trade.fee,
            trade.fee_asset
        );
    }
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**symbol** | **Option<String>** | Filter by trading pair | [optional] |

### Return type

[**Vec<Trade>**](Trade.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json