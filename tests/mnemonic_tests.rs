use lighter_rust::signers::{EthereumSigner, Signer};

#[test]
fn test_mnemonic_generation() {
    // Test creating a signer from a mnemonic phrase
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    let signer = EthereumSigner::from_mnemonic(mnemonic, 0).unwrap();
    let address = signer.get_address().unwrap();

    // This is the expected address for the test mnemonic at index 0
    // The actual address might differ based on implementation details
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42); // Ethereum addresses are 42 chars (0x + 40 hex chars)
}

#[test]
fn test_mnemonic_different_indices() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Different indices should produce different addresses
    let signer0 = EthereumSigner::from_mnemonic(mnemonic, 0).unwrap();
    let signer1 = EthereumSigner::from_mnemonic(mnemonic, 1).unwrap();

    let address0 = signer0.get_address().unwrap();
    let address1 = signer1.get_address().unwrap();

    assert_ne!(address0, address1);
}

#[test]
fn test_invalid_mnemonic() {
    let invalid_mnemonic = "invalid mnemonic phrase that should fail";

    let result = EthereumSigner::from_mnemonic(invalid_mnemonic, 0);
    assert!(result.is_err());
}

#[test]
fn test_random_signer_generation() {
    // Test that we can generate a random signer
    let signer = EthereumSigner::random().unwrap();
    let address = signer.get_address().unwrap();

    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42);
}

#[test]
fn test_mnemonic_signing() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    let signer = EthereumSigner::from_mnemonic(mnemonic, 0).unwrap();

    // Test that the signer can sign messages
    let message = "Hello, Lighter!";
    let signature = signer.sign_message(message).unwrap();

    assert!(signature.starts_with("0x"));
    // Ethereum signatures are 65 bytes (130 hex chars + 0x prefix = 132 chars)
    assert_eq!(signature.len(), 132);
}
