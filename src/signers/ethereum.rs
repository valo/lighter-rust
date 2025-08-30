use crate::error::{LighterError, Result};
use alloy::hex;
use alloy::primitives::B256;
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use bip39::{Language, Mnemonic};
use serde_json::json;
use sha3::{Digest, Keccak256};

pub trait Signer: std::fmt::Debug + Send + Sync {
    fn sign_message(&self, message: &str) -> Result<String>;
    fn get_address(&self) -> Result<String>;
}

#[derive(Debug, Clone)]
pub struct EthereumSigner {
    signer: PrivateKeySigner,
}

impl EthereumSigner {
    pub fn from_private_key(private_key: &str) -> Result<Self> {
        let private_key = private_key.trim_start_matches("0x");
        let signer =
            PrivateKeySigner::from_slice(&hex::decode(private_key).map_err(|e| {
                LighterError::Signing(format!("Invalid private key format: {}", e))
            })?)
            .map_err(|e| LighterError::Signing(format!("Invalid private key: {}", e)))?;

        Ok(Self { signer })
    }

    pub fn from_mnemonic(mnemonic_phrase: &str, account_index: u32) -> Result<Self> {
        use tiny_hderive::bip32::ExtendedPrivKey;

        // Parse the mnemonic
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_phrase)
            .map_err(|e| LighterError::Signing(format!("Invalid mnemonic: {}", e)))?;

        // Convert to seed
        let seed_bytes = mnemonic.to_seed("");

        // Derive the key at the specified path (m/44'/60'/0'/0/{account_index})
        let derivation_path = format!("m/44'/60'/0'/0/{}", account_index);
        let derived = ExtendedPrivKey::derive(&seed_bytes, derivation_path.as_str())
            .map_err(|e| LighterError::Signing(format!("Failed to derive key: {:?}", e)))?;

        // Get the private key bytes
        let private_key_bytes = derived.secret();

        // Create Alloy signer from the derived private key
        let signer = PrivateKeySigner::from_slice(&private_key_bytes)
            .map_err(|e| LighterError::Signing(format!("Failed to create signer: {}", e)))?;

        Ok(Self { signer })
    }

    pub fn random() -> Result<Self> {
        // Generate a random mnemonic using entropy
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut entropy = [0u8; 16]; // 128 bits for 12 word mnemonic
        rng.fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| LighterError::Signing(format!("Failed to generate mnemonic: {}", e)))?;

        println!("Generated mnemonic: {}", mnemonic);
        Self::from_mnemonic(&mnemonic.to_string(), 0)
    }

    fn hash_message(&self, message: &str) -> B256 {
        let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
        let mut hasher = Keccak256::new();
        hasher.update(prefix.as_bytes());
        hasher.update(message.as_bytes());
        B256::from_slice(&hasher.finalize())
    }
}

impl Signer for EthereumSigner {
    fn sign_message(&self, message: &str) -> Result<String> {
        let hash = self.hash_message(message);

        let signature = self
            .signer
            .sign_hash_sync(&hash)
            .map_err(|e| LighterError::Signing(format!("Failed to sign: {}", e)))?;

        Ok(format!("0x{}", hex::encode(signature.as_bytes())))
    }

    fn get_address(&self) -> Result<String> {
        Ok(format!("0x{:x}", self.signer.address()))
    }
}

pub fn sign_order_payload(
    signer: &dyn Signer,
    symbol: &str,
    side: &str,
    quantity: &str,
    price: Option<&str>,
    nonce: u64,
) -> Result<String> {
    let mut payload = json!({
        "symbol": symbol,
        "side": side,
        "quantity": quantity,
        "nonce": nonce,
    });

    if let Some(p) = price {
        payload["price"] = json!(p);
    }

    let message = serde_json::to_string(&payload)
        .map_err(|e| LighterError::Signing(format!("Failed to serialize payload: {}", e)))?;

    signer.sign_message(&message)
}

pub fn sign_cancel_payload(
    signer: &dyn Signer,
    order_id: Option<&str>,
    client_order_id: Option<&str>,
    symbol: Option<&str>,
    nonce: u64,
) -> Result<String> {
    let mut payload = json!({
        "nonce": nonce,
    });

    if let Some(id) = order_id {
        payload["order_id"] = json!(id);
    }

    if let Some(client_id) = client_order_id {
        payload["client_order_id"] = json!(client_id);
    }

    if let Some(sym) = symbol {
        payload["symbol"] = json!(sym);
    }

    let message = serde_json::to_string(&payload)
        .map_err(|e| LighterError::Signing(format!("Failed to serialize payload: {}", e)))?;

    signer.sign_message(&message)
}
