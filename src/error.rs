use thiserror::Error;

#[derive(Error, Debug)]
pub enum LighterError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tungstenite::Error),
    
    #[error("Signing error: {0}")]
    Signing(String),
    
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Authentication failed: {0}")]
    Auth(String),
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Invalid nonce: {0}")]
    Nonce(String),
    
    #[error("Account tier switch not allowed: {0}")]
    AccountTierSwitch(String),
    
    #[error("Invalid account state: {0}")]
    AccountState(String),
    
    #[error("Order validation failed: {0}")]
    OrderValidation(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, LighterError>;