# Account

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**id** | **String** | Unique account identifier | 
**address** | **String** | Ethereum address associated with the account | 
**tier** | [**AccountTier**](AccountTier.md) | Current account tier (Standard or Premium) | 
**created_at** | **DateTime<Utc>** | Account creation timestamp | 
**updated_at** | **DateTime<Utc>** | Last update timestamp | 
**balances** | [**Vec<Balance>**](Balance.md) | List of asset balances | 
**positions** | [**Vec<Position>**](Position.md) | List of open positions | 
**tier_switch_allowed_at** | **Option<DateTime<Utc>>** | Timestamp when tier switch is allowed | [optional]

## Example

```rust
use lighter_rust::Account;
use chrono::Utc;

// Example of Account structure
let account = Account {
    id: "acc_123456".to_string(),
    address: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
    tier: AccountTier::Premium,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    balances: vec![
        Balance {
            asset: "USDC".to_string(),
            total: "10000.50".to_string(),
            available: "9000.50".to_string(),
            locked: "1000.00".to_string(),
        },
    ],
    positions: vec![],
    tier_switch_allowed_at: None,
};
```

## Account Tiers

### Standard Account (Default)
- **Fees**: 0% maker / 0% taker
- **Latency**: 200ms maker/cancel, 300ms taker
- **Target**: Retail and latency-insensitive traders

### Premium Account
- **Fees**: 0.002% maker / 0.02% taker
- **Latency**: 0ms maker/cancel, 150ms taker
- **Target**: High-frequency traders (HFT)

## Tier Switching Rules

1. **No open positions**: All positions must be closed
2. **No open orders**: All orders must be cancelled or filled
3. **Time restriction**: Minimum 3-hour gap between switches
4. **Check eligibility**: Use `can_switch_tier()` method before attempting

## Related Methods

- [`get_account()`](AccountApi.md#get_account) - Retrieve account information
- [`change_account_tier()`](AccountApi.md#change_account_tier) - Switch account tier
- [`get_balances()`](AccountApi.md#get_balances) - Get asset balances
- [`get_positions()`](AccountApi.md#get_positions) - Get open positions