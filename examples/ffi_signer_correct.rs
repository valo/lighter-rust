use lighter_rust::error::Result;
use lighter_rust::models::common::OrderType;
use lighter_rust::models::order::TimeInForce;
use lighter_rust::signers::FFISigner;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Configuration
    let url = "https://api.testnet.lighter.xyz"; // or mainnet URL
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";
    let api_key_index = 0;
    let account_index = 0;

    // Create the FFI signer (this now calls CreateClient internally)
    let mut signer = FFISigner::new(url, private_key, api_key_index, account_index)?;

    println!("FFI Signer initialized successfully!");

    // Example 1: Sign a create order transaction
    println!("\n--- Creating and signing an order ---");

    let order_json = signer.sign_create_order(
        0,                // market_index (0 = BTC-USDC)
        1001,             // client_order_index
        1000000,          // base_amount (1 BTC in smallest units)
        650000,           // price ($65,000)
        false,            // is_ask (false = buy)
        OrderType::Limit, // order_type
        TimeInForce::Gtc, // time_in_force
        false,            // reduce_only
        0,                // trigger_price (0 for non-stop orders)
        1735689600,       // order_expiry (Unix timestamp)
        123456789,        // nonce
    )?;

    println!("Order Transaction JSON: {}", order_json);

    // Parse the JSON to see the structure
    let parsed: serde_json::Value = serde_json::from_str(&order_json)?;
    println!("Parsed transaction: {:#?}", parsed);

    // Example 2: Sign a cancel order transaction
    println!("\n--- Canceling an order ---");

    let cancel_json = signer.sign_cancel_order(
        0,             // market_index
        1002,          // client_cancel_index
        "order_12345", // order_id_to_cancel
        123456790,     // nonce
    )?;

    println!("Cancel Transaction JSON: {}", cancel_json);

    // Example 3: Sign a cancel all orders transaction
    println!("\n--- Canceling all orders ---");

    let cancel_all_json = signer.sign_cancel_all_orders(
        0,         // market_index
        1003,      // client_cancel_index
        123456791, // nonce
    )?;

    println!("Cancel All Transaction JSON: {}", cancel_all_json);

    // Example 4: Sign a transfer transaction
    // Note: For L1 transfers, you may need additional Ethereum signing
    println!("\n--- Creating a transfer ---");

    let transfer_json = signer.sign_transfer(
        "0xRecipientAddress123", // receiver address
        500000,                  // amount
        123456792,               // nonce
    )?;

    println!("Transfer Transaction JSON: {}", transfer_json);

    // Example 5: Sign a withdraw transaction
    println!("\n--- Creating a withdrawal ---");

    let withdraw_json = signer.sign_withdraw(
        "0xWithdrawAddress456", // receiver address
        1000000,                // amount
        123456793,              // nonce
    )?;

    println!("Withdraw Transaction JSON: {}", withdraw_json);

    // Example 6: Switch API key
    println!("\n--- Switching API key ---");

    signer.switch_api_key(1)?;
    println!("Switched to API key index 1");

    Ok(())
}

// Note: This corrected example demonstrates how to use the FFI signer with the native Go binaries.
//
// Key differences from the original implementation:
// 1. FFISigner::new() now takes a URL parameter and calls CreateClient internally
// 2. The StrOrErr structure has correct field names (str and err)
// 3. Functions return JSON strings directly from the Go library
// 4. Memory is properly managed using libc::free
// 5. No additional Ethereum signing is performed (the Go library handles signing)
