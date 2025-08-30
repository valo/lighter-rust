use crate::config::Config;
use crate::error::{LighterError, Result};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    config: Config,
}

impl ApiClient {
    pub fn new(config: Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(LighterError::Http)?;

        Ok(Self { client, config })
    }

    pub async fn get<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::GET, endpoint, None::<()>).await
    }

    pub async fn post<T, B>(&self, endpoint: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::POST, endpoint, body).await
    }

    pub async fn put<T, B>(&self, endpoint: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        self.request(Method::PUT, endpoint, body).await
    }

    pub async fn delete<T>(&self, endpoint: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        self.request(Method::DELETE, endpoint, None::<()>).await
    }

    async fn request<T, B>(&self, method: Method, endpoint: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let url = self.build_url(endpoint)?;
        let mut request_builder = self.client.request(method, url);

        if let Some(api_key) = &self.config.api_key {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        request_builder = request_builder.header("Content-Type", "application/json");
        request_builder = request_builder.header("User-Agent", "lighter-rust/0.1.0");

        if let Some(body) = body {
            request_builder = request_builder.json(&body);
        }

        let response = request_builder.send().await.map_err(LighterError::Http)?;
        self.handle_response(response).await
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await.map_err(LighterError::Http)?;

        if status.is_success() {
            serde_json::from_str(&body).map_err(LighterError::Json)
        } else {
            match status.as_u16() {
                429 => Err(LighterError::RateLimit),
                401 => Err(LighterError::Auth("Unauthorized".to_string())),
                _ => {
                    let error_message = serde_json::from_str::<serde_json::Value>(&body)
                        .ok()
                        .and_then(|v| v.get("message").and_then(|m| m.as_str().map(String::from)))
                        .unwrap_or_else(|| body);
                    
                    Err(LighterError::Api {
                        status: status.as_u16(),
                        message: error_message,
                    })
                }
            }
        }
    }

    fn build_url(&self, endpoint: &str) -> Result<Url> {
        let endpoint = endpoint.trim_start_matches('/');
        self.config
            .base_url
            .join(endpoint)
            .map_err(|e| LighterError::Config(format!("Invalid endpoint URL: {}", e)))
    }
}