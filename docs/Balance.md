# Balance

Asset balance information for an account.

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**asset** | **String** | Asset symbol (e.g., "USDC", "BTC") | 
**total** | **String** | Total balance amount | 
**available** | **String** | Available balance for trading | 
**locked** | **String** | Balance locked in orders or positions | 

## Example

```rust
use lighter_rust::Balance;

let balance = Balance {
    asset: "USDC".to_string(),
    total: "10000.50".to_string(),
    available: "8500.50".to_string(),
    locked: "1500.00".to_string(),
};

// Calculate percentage locked
let total: f64 = balance.total.parse().unwrap();
let locked: f64 = balance.locked.parse().unwrap();
let locked_percentage = (locked / total) * 100.0;
println!("{}% of {} is locked", locked_percentage, balance.asset);
```

## Balance States

### Total Balance
The complete balance of an asset in the account, including both available and locked amounts.

### Available Balance
The portion of the balance that can be used for:
- Placing new orders
- Opening new positions
- Withdrawals

### Locked Balance
The portion of the balance that is currently:
- Reserved for open orders
- Used as margin for positions
- Pending settlement

## Balance Updates

Balances are updated in real-time when:
- Orders are placed (available decreases, locked increases)
- Orders are cancelled (available increases, locked decreases)
- Orders are filled (locked decreases)
- Deposits are confirmed (total and available increase)
- Withdrawals are processed (total and available decrease)

## Cross-Margin vs Isolated Margin

### Cross-Margin Mode
- All balances contribute to margin requirements
- Losses in one position can be offset by profits in another
- More capital efficient

### Isolated Margin Mode
- Each position has dedicated margin
- Losses are limited to the position's margin
- Better risk isolation

## Related Methods

- [`get_balances()`](AccountApi.md#get_balances) - Retrieve all balances
- [`get_account()`](AccountApi.md#get_account) - Get full account info including balances