use crate::error::{LighterError, Result};
use alloy::hex;
use alloy::primitives::B256;
use alloy::signers::{local::PrivateKeySigner, SignerSync};
use bip39::{Language, Mnemonic};
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
        let decoded = hex::decode(private_key)
            .map_err(|e| LighterError::Signing(e.to_string()))?;
        let signer = PrivateKeySigner::from_slice(&decoded)
            .map_err(|e| LighterError::Signing(e.to_string()))?;

        Ok(Self { signer })
    }

    pub fn from_mnemonic(mnemonic_phrase: &str, account_index: u32) -> Result<Self> {
        use tiny_hderive::bip32::ExtendedPrivKey;

        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_phrase)
            .map_err(|e| LighterError::Signing(e.to_string()))?;

        let seed_bytes = mnemonic.to_seed("");
        let derivation_path = format!("m/44'/60'/0'/0/{}", account_index);
        let derived = ExtendedPrivKey::derive(&seed_bytes, derivation_path.as_str())
            .map_err(|e| LighterError::Signing(format!("{:?}", e)))?;

        let private_key_bytes = derived.secret();
        let signer = PrivateKeySigner::from_slice(&private_key_bytes)
            .map_err(|e| LighterError::Signing(e.to_string()))?;

        Ok(Self { signer })
    }

    pub fn random() -> Result<Self> {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut entropy = [0u8; 16];
        rng.fill_bytes(&mut entropy);

        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| LighterError::Signing(e.to_string()))?;

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
        let signature = self.signer.sign_hash_sync(&hash)
            .map_err(|e| LighterError::Signing(e.to_string()))?;
        Ok(format!("0x{}", hex::encode(signature.as_bytes())))
    }

    fn get_address(&self) -> Result<String> {
        Ok(format!("0x{:x}", self.signer.address()))
    }
}