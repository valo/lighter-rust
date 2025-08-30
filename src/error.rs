use thiserror::Error;

#[derive(Error, Debug)]
pub enum LighterError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] Box<reqwest::Error>),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] Box<tungstenite::Error>),

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = LighterError::Auth("Invalid API key".to_string());
        assert_eq!(error.to_string(), "Authentication failed: Invalid API key");

        let error = LighterError::RateLimit;
        assert_eq!(error.to_string(), "Rate limit exceeded");

        let error = LighterError::Api {
            status: 404,
            message: "Order not found".to_string(),
        };
        assert_eq!(error.to_string(), "API error: 404 - Order not found");
    }

    #[tokio::test]
    async fn test_error_from_reqwest() {
        // This tests the automatic conversion from reqwest::Error
        let url = "http://invalid-url-that-doesnt-exist-12345.com";
        let client = reqwest::Client::new();
        let result = client.get(url).send().await;

        if let Err(e) = result {
            let lighter_error: LighterError = Box::new(e).into();
            assert!(matches!(lighter_error, LighterError::Http(_)));
        }
    }

    #[test]
    fn test_error_from_json() {
        let invalid_json = "{ invalid json }";
        let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(invalid_json);

        if let Err(e) = result {
            let lighter_error: LighterError = e.into();
            assert!(matches!(lighter_error, LighterError::Json(_)));
        }
    }

    #[test]
    fn test_result_type_alias() {
        fn test_function() -> Result<String> {
            Ok("success".to_string())
        }

        let result = test_function();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");

        fn test_error_function() -> Result<String> {
            Err(LighterError::Unknown("test error".to_string()))
        }

        let result = test_error_function();
        assert!(result.is_err());
    }
}
