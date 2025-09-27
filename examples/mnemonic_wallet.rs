use lighter_rust::{
    init_logging,
    signers::{EthereumSigner, Signer},
    Config, LighterClient,
};

/// Example demonstrating how to use mnemonic phrases with the Lighter SDK
///
/// This example shows:
/// - Creating a wallet from a mnemonic phrase
/// - Using different account indices
/// - Generating a random wallet
/// - Initializing the LighterClient with a mnemonic

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    println!("=== Mnemonic Wallet Example ===\n");

    // Example 1: Create signer from a known mnemonic
    // NEVER use this mnemonic in production - it's a well-known test mnemonic
    let test_mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    println!("1. Creating wallet from test mnemonic...");
    let signer = EthereumSigner::from_mnemonic(test_mnemonic, 0)?;
    let address = signer.get_address()?;
    println!("   Account 0 address: {}", address);

    // Example 2: Use different account indices from the same mnemonic
    println!("\n2. Deriving multiple accounts from same mnemonic...");
    for i in 0..5 {
        let signer = EthereumSigner::from_mnemonic(test_mnemonic, i)?;
        let address = signer.get_address()?;
        println!("   Account {} address: {}", i, address);
    }

    // Example 3: Generate a random mnemonic (for new wallets)
    println!("\n3. Generating a random wallet...");
    let random_signer = EthereumSigner::random()?;
    let random_address = random_signer.get_address()?;
    println!("   Random wallet address: {}", random_address);
    println!("   (Note: Save the mnemonic securely to restore this wallet!)");

    // Example 4: Initialize LighterClient with mnemonic
    println!("\n4. Initializing Lighter client with mnemonic...");

    let config = Config::new().with_api_key("your-api-key").with_timeout(30);

    // Create client using mnemonic (account index 0)
    let _client = LighterClient::from_mnemonic(config, test_mnemonic, 0)?;

    println!("   Client initialized successfully!");

    // Example 5: Sign a message with mnemonic-derived wallet
    println!("\n5. Signing a message...");
    let message = "Hello from Lighter SDK!";
    let signature = signer.sign_message(message)?;
    println!("   Message: {}", message);
    println!("   Signature: {}", signature);

    // Example 6: Production usage pattern
    println!("\n6. Production Usage Pattern:");
    println!("   - Store mnemonic securely (e.g., environment variable, secure vault)");
    println!("   - Never hardcode mnemonics in source code");
    println!("   - Use hardware wallets for high-value operations");
    println!("   - Consider using different account indices for different purposes");

    // Example of loading from environment (commented out for demo)
    /*
    let mnemonic = std::env::var("LIGHTER_MNEMONIC")
        .expect("LIGHTER_MNEMONIC environment variable not set");
    let account_index = std::env::var("LIGHTER_ACCOUNT_INDEX")
        .unwrap_or_else(|_| "0".to_string())
        .parse::<u32>()?;

    let production_client = LighterClient::from_mnemonic(
        config,
        &mnemonic,
        account_index
    )?;
    */

    println!("\n=== Mnemonic Example Complete ===");

    Ok(())
}
