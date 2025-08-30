use lighter_rust::signers::{EthereumSigner, Signer};
use lighter_rust::{sign_cancel_payload, sign_order_payload};

#[test]
fn test_ethereum_signer_creation() {
    // Test with a valid private key (test key, not real)
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key);
    assert!(signer.is_ok());

    // Test with invalid private key
    let invalid_key = "invalid_key";
    let signer = EthereumSigner::from_private_key(invalid_key);
    assert!(signer.is_err());
}

#[test]
fn test_address_derivation() {
    // Known test private key and expected address
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    let address = signer.get_address().unwrap();
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42); // 0x + 40 hex chars
}

#[test]
fn test_message_signing() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    let message = "Hello, Lighter!";
    let signature = signer.sign_message(message).unwrap();

    // Signature should be hex string starting with 0x
    assert!(signature.starts_with("0x"));
    // Signature should be 132 chars (0x + 130 hex chars for 65 bytes)
    assert!(signature.len() >= 130);
}

#[test]
fn test_deterministic_signing() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    let message = "Deterministic message";
    let sig1 = signer.sign_message(message).unwrap();
    let sig2 = signer.sign_message(message).unwrap();

    // Same message with same key should produce same signature
    assert_eq!(sig1, sig2);

    // Different message should produce different signature
    let sig3 = signer.sign_message("Different message").unwrap();
    assert_ne!(sig1, sig3);
}

#[test]
fn test_sign_order_payload() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    let signature =
        sign_order_payload(&signer, "BTC-USDC", "BUY", "0.1", Some("45000"), 12345678).unwrap();

    assert!(signature.starts_with("0x"));
    assert!(signature.len() >= 130);

    // Test without price (market order)
    let market_sig =
        sign_order_payload(&signer, "BTC-USDC", "SELL", "0.5", None, 12345679).unwrap();

    assert!(market_sig.starts_with("0x"));
    assert_ne!(signature, market_sig); // Different payloads
}

#[test]
fn test_sign_cancel_payload() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    // Test with order_id
    let sig1 = sign_cancel_payload(&signer, Some("order123"), None, None, 12345680).unwrap();

    assert!(sig1.starts_with("0x"));

    // Test with client_order_id
    let sig2 = sign_cancel_payload(&signer, None, Some("client456"), None, 12345681).unwrap();

    assert!(sig2.starts_with("0x"));
    assert_ne!(sig1, sig2);

    // Test with symbol (cancel all for symbol)
    let sig3 = sign_cancel_payload(&signer, None, None, Some("BTC-USDC"), 12345682).unwrap();

    assert!(sig3.starts_with("0x"));
    assert_ne!(sig2, sig3);
}

#[test]
fn test_nonce_in_signatures() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    // Same payload with different nonce should produce different signatures
    let sig1 =
        sign_order_payload(&signer, "BTC-USDC", "BUY", "1.0", Some("50000"), 100000).unwrap();

    let sig2 = sign_order_payload(
        &signer,
        "BTC-USDC",
        "BUY",
        "1.0",
        Some("50000"),
        100001, // Different nonce
    )
    .unwrap();

    assert_ne!(sig1, sig2);
}

#[test]
fn test_json_payload_format() {
    // Test that the payload being signed is valid JSON
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    // This indirectly tests that the JSON serialization works
    let result = sign_order_payload(
        &signer,
        "ETH-USDC",
        "SELL",
        "2.5",
        Some("3000.50"),
        999999999,
    );

    assert!(result.is_ok());
}

#[test]
fn test_signer_trait_implementation() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    // Test that EthereumSigner implements Signer trait
    let signer_ref: &dyn Signer = &signer;

    let address = signer_ref.get_address().unwrap();
    assert!(address.starts_with("0x"));

    let signature = signer_ref.sign_message("Test message").unwrap();
    assert!(signature.starts_with("0x"));
}

#[test]
fn test_hex_encoding() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    let signature = signer.sign_message("test").unwrap();

    // Remove 0x prefix and check if remaining is valid hex
    let hex_part = &signature[2..];
    assert!(hex_part.chars().all(|c| c.is_ascii_hexdigit()));

    // Should be even number of hex characters (pairs of bytes)
    assert_eq!(hex_part.len() % 2, 0);
}
