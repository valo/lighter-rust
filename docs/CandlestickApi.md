# CandlestickApi

All URIs are relative to *https://api.lighter.xyz*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_candlesticks**](CandlestickApi.md#get_candlesticks) | **GET** /candlesticks | Get candlestick data
[**get_market_stats**](CandlestickApi.md#get_market_stats) | **GET** /market/stats/{symbol} | Get market statistics
[**get_all_market_stats**](CandlestickApi.md#get_all_market_stats) | **GET** /market/stats | Get all market statistics
[**get_ticker**](CandlestickApi.md#get_ticker) | **GET** /ticker/{symbol} | Get ticker
[**get_all_tickers**](CandlestickApi.md#get_all_tickers) | **GET** /ticker | Get all tickers
[**get_order_book**](CandlestickApi.md#get_order_book) | **GET** /orderbook/{symbol} | Get order book

## get_candlesticks

> Vec<Candlestick> get_candlesticks(symbol, interval, start_time, end_time, limit)

Get candlestick data

Returns OHLCV candlestick data for a trading pair.

### Example

```rust
use lighter_rust::{LighterClient, Config, CandlestickInterval};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new_read_only(config)?;
    
    // Get last 100 1-hour candles for BTC-USDC
    let candlesticks = client.market_data().get_candlesticks(
        "BTC-USDC",
        CandlestickInterval::OneHour,
        Some(Utc::now() - Duration::hours(100)),
        Some(Utc::now()),
        Some(100),
    ).await?;
    
    for candle in candlesticks {
        println!("Time: {} O:{} H:{} L:{} C:{} V:{}", 
            candle.open_time,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.volume
        );
    }
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**symbol** | **String** | Trading pair symbol | [required] |
**interval** | [**CandlestickInterval**](CandlestickInterval.md) | Candlestick interval | [required] |
**start_time** | **Option<DateTime<Utc>>** | Start time | [optional] |
**end_time** | **Option<DateTime<Utc>>** | End time | [optional] |
**limit** | **Option<u32>** | Max number of results | [optional] [default to 500]

### Return type

[**Vec<Candlestick>**](Candlestick.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_market_stats

> MarketStats get_market_stats(symbol)

Get market statistics

Returns 24-hour market statistics for a trading pair.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new_read_only(config)?;
    
    let stats = client.market_data()
        .get_market_stats("BTC-USDC")
        .await?;
    
    println!("Symbol: {}", stats.symbol);
    println!("Last Price: {}", stats.last_price);
    println!("24h Change: {}%", stats.price_change_percent);
    println!("24h Volume: {}", stats.volume);
    println!("24h High: {}", stats.high_price);
    println!("24h Low: {}", stats.low_price);
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**symbol** | **String** | Trading pair symbol | [required] |

### Return type

[**MarketStats**](MarketStats.md)

### Authorization

None

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_all_market_stats

> Vec<MarketStats> get_all_market_stats()

Get all market statistics

Returns 24-hour market statistics for all trading pairs.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    
    let client = LighterClient::new_read_only(config)?;
    
    let all_stats = client.market_data()
        .get_all_market_stats()
        .await?;
    
    for stats in all_stats {
        println!("{}: {} ({}%)", 
            stats.symbol,
            stats.last_price,
            stats.price_change_percent
        );
    }
    
    Ok(())
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<MarketStats>**](MarketStats.md)

### Authorization

None

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_ticker

> Ticker get_ticker(symbol)

Get ticker

Returns current ticker information for a trading pair.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    
    let client = LighterClient::new_read_only(config)?;
    
    let ticker = client.market_data()
        .get_ticker("BTC-USDC")
        .await?;
    
    println!("BTC-USDC Price: {}", ticker.price);
    println!("Updated: {}", ticker.timestamp);
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**symbol** | **String** | Trading pair symbol | [required] |

### Return type

[**Ticker**](Ticker.md)

### Authorization

None

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_all_tickers

> Vec<Ticker> get_all_tickers()

Get all tickers

Returns current ticker information for all trading pairs.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    
    let client = LighterClient::new_read_only(config)?;
    
    let tickers = client.market_data()
        .get_all_tickers()
        .await?;
    
    for ticker in tickers {
        println!("{}: {}", ticker.symbol, ticker.price);
    }
    
    Ok(())
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<Ticker>**](Ticker.md)

### Authorization

None

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_order_book

> OrderBook get_order_book(symbol, depth)

Get order book

Returns the current order book for a trading pair.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new();
    
    let client = LighterClient::new_read_only(config)?;
    
    // Get order book with depth of 20 levels
    let order_book = client.market_data()
        .get_order_book("BTC-USDC", Some(20))
        .await?;
    
    println!("Best Bid: {} @ {}", 
        order_book.bids[0].quantity,
        order_book.bids[0].price
    );
    
    println!("Best Ask: {} @ {}", 
        order_book.asks[0].quantity,
        order_book.asks[0].price
    );
    
    let spread = order_book.asks[0].price.parse::<f64>()? 
        - order_book.bids[0].price.parse::<f64>()?;
    println!("Spread: {:.2}", spread);
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**symbol** | **String** | Trading pair symbol | [required] |
**depth** | **Option<u32>** | Order book depth | [optional] [default to 20]

### Return type

[**OrderBook**](OrderBook.md)

### Authorization

None

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json