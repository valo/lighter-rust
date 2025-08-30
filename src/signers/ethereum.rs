use crate::error::{LighterError, Result};
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use alloy::primitives::B256;
use alloy::hex;
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
        let signer = PrivateKeySigner::from_slice(&hex::decode(private_key)
            .map_err(|e| LighterError::Signing(format!("Invalid private key format: {}", e)))?)
            .map_err(|e| LighterError::Signing(format!("Invalid private key: {}", e)))?;
        
        Ok(Self { signer })
    }
    
    pub fn from_mnemonic(_mnemonic: &str, _index: u32) -> Result<Self> {
        // For now, this is a placeholder - Alloy v0.5 may not have from_mnemonic
        // In a real implementation, you'd derive the private key from the mnemonic
        return Err(LighterError::Signing("Mnemonic support not implemented yet".to_string()));
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
        
        let signature = self.signer
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