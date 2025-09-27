//! # Lighter Rust SDK
//!
//! A Rust SDK for the Lighter trading platform, providing async API access to trading operations,
//! account management, and real-time market data.
//!
//! ## Features
//!
//! - **Async/await support** - Built with Tokio for high-performance async operations
//! - **Type-safe API** - Comprehensive Rust types for all API responses
//! - **WebSocket & REST** - Support for both REST API and real-time WebSocket feeds
//! - **Account management** - Handle Standard and Premium account tiers
//! - **Trading operations** - Place orders, manage positions, query market data
//! - **Ethereum signing** - Built-in support for Ethereum-compatible wallet signing
//!
//! ## Quick Start
//!
//! ```no_run
//! use lighter_rust::{LighterClient, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::new().with_api_key("your-api-key");
//!     let client = LighterClient::new(config, "your-private-key")?;
//!     
//!     // Get account info
//!     let account = client.account().get_account().await?;
//!     println!("Account: {:?}", account);
//!     
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod logging;
pub mod models;
pub mod nonce;
pub mod signers;

// Re-export specific items to avoid ambiguous glob re-exports
pub use api::{account::AccountApi, candlestick::CandlestickApi, order::OrderApi, transaction::TransactionApi};
pub use client::{api_client::ApiClient, signer_client::SignerClient, ws_client::WebSocketClient};
pub use config::Config;
pub use error::{LighterError, Result};
pub use logging::{init_logging, init_logging_with_filter};
// Re-export models modules individually
pub use models::common::*;
pub use models::order::{Order, CreateOrderRequest, TimeInForce};
pub use models::account::Account;
pub use signers::{ethereum::*, ffi::*};

/// Main client for interacting with the Lighter API
#[derive(Debug)]
pub struct LighterClient {
    account_api: AccountApi,
    order_api: OrderApi,
    transaction_api: TransactionApi,
    candlestick_api: CandlestickApi,
    ws_client: WebSocketClient,
}

impl LighterClient {
    /// Create a new Lighter client with the given configuration and private key
    pub fn new(config: Config, private_key: &str) -> Result<Self> {
        let api_client = ApiClient::new(config.clone())?;
        let signer_client = SignerClient::with_ethereum_signer(api_client, private_key)?;
        let ws_client = WebSocketClient::new(config);

        Ok(Self {
            account_api: AccountApi::new(signer_client.clone()),
            order_api: OrderApi::new(signer_client.clone()),
            transaction_api: TransactionApi::new(signer_client.clone()),
            candlestick_api: CandlestickApi::new(signer_client),
            ws_client,
        })
    }

    /// Create a new Lighter client with a mnemonic phrase
    pub fn from_mnemonic(config: Config, mnemonic: &str, account_index: u32) -> Result<Self> {
        let api_client = ApiClient::new(config.clone())?;
        let ethereum_signer = signers::EthereumSigner::from_mnemonic(mnemonic, account_index)?;
        let signer_client = SignerClient::new(api_client, std::sync::Arc::new(ethereum_signer));
        let ws_client = WebSocketClient::new(config);

        Ok(Self {
            account_api: AccountApi::new(signer_client.clone()),
            order_api: OrderApi::new(signer_client.clone()),
            transaction_api: TransactionApi::new(signer_client.clone()),
            candlestick_api: CandlestickApi::new(signer_client),
            ws_client,
        })
    }

    /// Create a new client with just an API key (no signing capabilities)
    pub fn new_read_only(config: Config) -> Result<Self> {
        let api_client = ApiClient::new(config.clone())?;
        let ws_client = WebSocketClient::new(config);

        // For read-only client, we'll use a dummy signer that will error on signing operations
        let dummy_signer = signers::EthereumSigner::from_private_key(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )?;
        let signer_client = SignerClient::new(api_client, std::sync::Arc::new(dummy_signer));

        Ok(Self {
            account_api: AccountApi::new(signer_client.clone()),
            order_api: OrderApi::new(signer_client.clone()),
            transaction_api: TransactionApi::new(signer_client.clone()),
            candlestick_api: CandlestickApi::new(signer_client),
            ws_client,
        })
    }

    /// Access account-related API endpoints
    pub fn account(&self) -> &AccountApi {
        &self.account_api
    }

    /// Access order-related API endpoints
    pub fn orders(&self) -> &OrderApi {
        &self.order_api
    }

    /// Access transaction-related API endpoints  
    pub fn transactions(&self) -> &TransactionApi {
        &self.transaction_api
    }

    /// Access market data API endpoints
    pub fn market_data(&self) -> &CandlestickApi {
        &self.candlestick_api
    }

    /// Access WebSocket client
    pub fn websocket(&mut self) -> &mut WebSocketClient {
        &mut self.ws_client
    }
}
