# AccountApi

All URIs are relative to *https://api.lighter.xyz*

Method | HTTP request | Description
------------- | ------------- | -------------
[**get_account**](AccountApi.md#get_account) | **GET** /account | Get account information
[**get_account_stats**](AccountApi.md#get_account_stats) | **GET** /account/stats | Get account statistics
[**change_account_tier**](AccountApi.md#change_account_tier) | **POST** /account/change-tier | Change account tier
[**get_balances**](AccountApi.md#get_balances) | **GET** /account/balances | Get account balances
[**get_positions**](AccountApi.md#get_positions) | **GET** /account/positions | Get open positions

## get_account

> Account get_account()

Get account information

Retrieves the current account information including tier, balances, and positions.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    let account = client.account().get_account().await?;
    println!("Account ID: {}", account.id);
    println!("Account Tier: {:?}", account.tier);
    
    Ok(())
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**Account**](Account.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_account_stats

> AccountStats get_account_stats()

Get account statistics

Returns trading statistics for the account including volume, fees, and performance metrics.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    let stats = client.account().get_account_stats().await?;
    println!("Total Volume: {}", stats.total_volume);
    println!("Total Trades: {}", stats.total_trades);
    println!("Win Rate: {}%", stats.win_rate);
    
    Ok(())
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**AccountStats**](AccountStats.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## change_account_tier

> change_account_tier(target_tier)

Change account tier

Switch between Standard and Premium account tiers. Requirements:
- No open positions
- No open orders
- Minimum 3-hour gap between switches

### Example

```rust
use lighter_rust::{LighterClient, Config, AccountTier};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    // Check if tier switch is allowed
    if client.account().can_switch_tier().await? {
        // Switch to Premium tier
        client.account().change_account_tier(AccountTier::Premium).await?;
        println!("Successfully switched to Premium tier");
    } else {
        println!("Cannot switch tier at this time");
    }
    
    Ok(())
}
```

### Parameters

Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**target_tier** | [**AccountTier**](AccountTier.md) | Target account tier | [required] |

### Return type

(empty response body)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth), [SignatureAuth](../README.md#SignatureAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

## get_balances

> Vec<Balance> get_balances()

Get account balances

Returns all asset balances for the account.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    let balances = client.account().get_balances().await?;
    
    for balance in balances {
        println!("{}: {} (Available: {})", 
            balance.asset, 
            balance.total, 
            balance.available
        );
    }
    
    Ok(())
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<Balance>**](Balance.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

## get_positions

> Vec<Position> get_positions()

Get open positions

Returns all open positions for the account.

### Example

```rust
use lighter_rust::{LighterClient, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new()
        .with_api_key("your-api-key");
    
    let client = LighterClient::new(config, "your-private-key")?;
    
    let positions = client.account().get_positions().await?;
    
    for position in positions {
        println!("{} {} @ {} - PnL: {}", 
            position.symbol,
            position.size,
            position.entry_price,
            position.unrealized_pnl
        );
    }
    
    Ok(())
}
```

### Parameters

This endpoint does not need any parameter.

### Return type

[**Vec<Position>**](Position.md)

### Authorization

[ApiKeyAuth](../README.md#ApiKeyAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json