use crate::client::ApiClient;
use crate::error::Result;
use crate::nonce::NonceManager;
use crate::signers::{EthereumSigner, Signer};
use std::sync::Arc;

#[derive(Debug)]
pub struct SignerClient {
    api_client: ApiClient,
    signer: Arc<dyn Signer + Send + Sync>,
    nonce_manager: Arc<NonceManager>,
}

impl SignerClient {
    pub fn new(api_client: ApiClient, signer: Arc<dyn Signer + Send + Sync>) -> Self {
        Self {
            api_client,
            signer,
            nonce_manager: Arc::new(NonceManager::new()),
        }
    }

    pub fn with_ethereum_signer(api_client: ApiClient, private_key: &str) -> Result<Self> {
        let ethereum_signer = EthereumSigner::from_private_key(private_key)?;
        let signer: Arc<dyn Signer + Send + Sync> = Arc::new(ethereum_signer);

        Ok(Self::new(api_client, signer))
    }

    pub fn api_client(&self) -> &ApiClient {
        &self.api_client
    }

    pub fn signer(&self) -> &Arc<dyn Signer + Send + Sync> {
        &self.signer
    }

    pub fn generate_nonce(&self) -> Result<u64> {
        self.nonce_manager.generate()
    }

    pub fn get_address(&self) -> Result<String> {
        self.signer.get_address()
    }
}

impl Clone for SignerClient {
    fn clone(&self) -> Self {
        Self {
            api_client: self.api_client.clone(),
            signer: self.signer.clone(),
            nonce_manager: self.nonce_manager.clone(),
        }
    }
}
