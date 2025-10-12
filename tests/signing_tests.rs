use lighter_rust::models::common::{OrderType, Side};
use lighter_rust::models::order::TimeInForce;
use lighter_rust::models::AccountTier;
use lighter_rust::signers::{
    account_tier_signature_message, order_signature_message, sign_account_tier_payload,
    sign_order_payload, EthereumSigner, Signer,
};

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

// Note: Order signing tests removed as they now use FFISigner

#[test]
fn test_nonce_in_signatures() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    // Same payload with different nonce should produce different signatures
    let sig1 = sign_order_payload(
        &signer,
        "BTC-USDC",
        Side::Buy,
        OrderType::Limit,
        "1.0",
        Some("50000"),
        None,
        None,
        TimeInForce::Gtc,
        None,
        None,
        100000,
    )
    .unwrap();

    let sig2 = sign_order_payload(
        &signer,
        "BTC-USDC",
        Side::Buy,
        OrderType::Limit,
        "1.0",
        Some("50000"),
        None,
        None,
        TimeInForce::Gtc,
        None,
        None,
        100001,
    )
    .unwrap();

    assert_ne!(sig1, sig2);
}

#[test]
fn test_json_payload_format() {
    // Test that the payload being signed is valid JSON
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    let message = order_signature_message(
        "ETH-USDC",
        Side::Sell,
        OrderType::Limit,
        "2.5",
        Some("3000.50"),
        None,
        None,
        TimeInForce::Gtc,
        None,
        None,
        999_999_999,
    )
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&message).unwrap();
    assert_eq!(parsed["symbol"], "ETH-USDC");
    assert_eq!(parsed["side"], "SELL");
    assert_eq!(parsed["price"], "3000.50");
    assert_eq!(parsed["nonce"], 999_999_999);

    let signature = sign_order_payload(
        &signer,
        "ETH-USDC",
        Side::Sell,
        OrderType::Limit,
        "2.5",
        Some("3000.50"),
        None,
        None,
        TimeInForce::Gtc,
        None,
        None,
        999_999_999,
    )
    .unwrap();
    assert!(signature.starts_with("0x"));
}

#[test]
fn test_order_payload_includes_optional_fields() {
    let message = order_signature_message(
        "BTC-USDC",
        Side::Sell,
        OrderType::StopLoss,
        "3.0",
        Some("45000"),
        Some("44000"),
        Some("client-123"),
        TimeInForce::Day,
        Some(true),
        Some(false),
        424242,
    )
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&message).unwrap();

    assert_eq!(parsed["symbol"], "BTC-USDC");
    assert_eq!(parsed["orderType"], "STOP_LOSS");
    assert_eq!(parsed["clientOrderId"], "client-123");
    assert_eq!(parsed["timeInForce"], "DAY");
    assert_eq!(parsed["postOnly"], true);
    assert_eq!(parsed["reduceOnly"], false);
    assert_eq!(parsed["stopPrice"], "44000");
}

#[test]
fn test_known_order_signature_vector() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();

    let signature = sign_order_payload(
        &signer,
        "BTC-USDC",
        Side::Buy,
        OrderType::Limit,
        "1.5",
        Some("35000"),
        None,
        None,
        TimeInForce::Gtc,
        Some(true),
        Some(false),
        1_725_000_000_000,
    )
    .unwrap();
    let expected = "0xc9930a8ae4690361180eef3efd77cf6979fa6d4c65863d74fd6e8a1c251cbec970081b1ea899aa97a179195f70a2b175b0584b906a9103e5dba92094ed96ffcd1c";
    assert_eq!(signature, expected);
}

#[test]
fn test_account_tier_payload_signing() {
    let private_key = "0000000000000000000000000000000000000000000000000000000000000001";
    let signer = EthereumSigner::from_private_key(private_key).unwrap();
    let nonce = 123_456_789_u64;

    let helper_signature = sign_account_tier_payload(&signer, AccountTier::Premium, nonce).unwrap();
    let manual_message = account_tier_signature_message(AccountTier::Premium, nonce).unwrap();
    let manual_signature = signer.sign_message(&manual_message).unwrap();

    assert_eq!(helper_signature, manual_signature);
    assert!(helper_signature.starts_with("0x"));
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
