use lighter_rust::error::Result;
use lighter_rust::models::common::OrderType;
use lighter_rust::models::order::TimeInForce;
use lighter_rust::signers::ffi::{FFISigner, SignedTransaction};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Your private key (keep this secure!)
    let private_key = "0x1234567890123456789012345678901234567890123456789012345678901234";

    // Create the FFI signer
    // api_key_index: Your API key index from Lighter
    // account_index: Your account index (usually 0 for first account)
    let signer = FFISigner::new(
        private_key,
        0, // api_key_index
        0, // account_index
    )?;

    println!("FFI Signer initialized successfully!");

    // Example 1: Sign a create order transaction
    println!("\n--- Creating and signing an order ---");

    let signed_order = signer.sign_create_order(
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

    print_transaction(&signed_order);

    // Example 2: Sign a cancel order transaction
    println!("\n--- Canceling an order ---");

    let signed_cancel = signer.sign_cancel_order(
        0,             // market_index
        1002,          // client_cancel_index
        "order_12345", // order_id_to_cancel
        123456790,     // nonce
    )?;

    print_transaction(&signed_cancel);

    // Example 3: Sign a cancel all orders transaction
    println!("\n--- Canceling all orders ---");

    let signed_cancel_all = signer.sign_cancel_all_orders(
        0,         // market_index
        1003,      // client_cancel_index
        123456791, // nonce
    )?;

    print_transaction(&signed_cancel_all);

    // Example 4: Sign a transfer transaction
    println!("\n--- Creating a transfer ---");

    let signed_transfer = signer.sign_transfer(
        "0xRecipientAddress123", // receiver address
        500000,                  // amount
        123456792,               // nonce
    )?;

    print_transaction(&signed_transfer);

    // Example 5: Sign a withdraw transaction
    println!("\n--- Creating a withdrawal ---");

    let signed_withdraw = signer.sign_withdraw(
        "0xWithdrawAddress456", // receiver address
        1000000,                // amount
        123456793,              // nonce
    )?;

    print_transaction(&signed_withdraw);

    Ok(())
}

fn print_transaction(tx: &SignedTransaction) {
    println!("Transaction ID: {}", tx.id);
    println!("Sequence: {}", tx.sequence);
    println!("Message to Sign: {}", tx.message_to_sign);
    if let Some(sig) = &tx.signature {
        println!("Signature: {}", sig);
    }
    println!(
        "Transaction Details: {}",
        serde_json::to_string_pretty(&tx.transaction).unwrap()
    );
}

// Note: This example demonstrates how to use the FFI signer with the native Go binaries.
// The FFI signer combines two cryptographic approaches:
// 1. The Go binary generates properly formatted transaction messages using Goldilocks/Schnorr
// 2. Your Ethereum private key signs these messages using standard Ethereum signing
//
// This hybrid approach ensures compatibility with Lighter's custom cryptographic requirements
// while maintaining the security of your Ethereum private key.
