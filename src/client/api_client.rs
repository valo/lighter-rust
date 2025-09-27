use crate::config::Config;
use crate::error::{LighterError, Result};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn};
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
            .pool_max_idle_per_host(10) // Keep up to 10 idle connections per host
            .pool_idle_timeout(Duration::from_secs(90)) // Keep connections alive for 90 seconds
            .tcp_keepalive(Duration::from_secs(60)) // TCP keepalive every 60 seconds
            .tcp_nodelay(true) // Disable Nagle's algorithm for lower latency
            .http1_only() // Force HTTP/1.1 for compatibility with Lighter API
            .connection_verbose(false)
            .build()
            .map_err(|e| LighterError::Http(Box::new(e)))?;

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
        B: Serialize + Clone,
    {
        self.request(Method::POST, endpoint, body).await
    }

    pub async fn put<T, B>(&self, endpoint: &str, body: Option<B>) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + Clone,
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
        B: Serialize + Clone,
    {
        let url = self.build_url(endpoint)?;
        let mut retries = 0;
        let max_retries = self.config.max_retries;

        loop {
            let mut request_builder = self.client.request(method.clone(), url.clone());

            if let Some(api_key) = &self.config.api_key {
                request_builder =
                    request_builder.header("Authorization", format!("Bearer {}", api_key));
            }

            request_builder = request_builder.header("Content-Type", "application/json");
            request_builder = request_builder.header("User-Agent", "lighter-rust/0.1.0");

            if let Some(ref body) = body {
                request_builder = request_builder.json(body);
            }

            debug!("Sending {} request to {}", method, url);

            match request_builder.send().await {
                Ok(response) => {
                    let status = response.status();

                    // Check if we should retry based on status code
                    if (status.as_u16() == 429 || status.is_server_error()) && retries < max_retries
                    {
                        retries += 1;
                        let delay = self.calculate_backoff_delay(retries);

                        warn!(
                            "Request failed with status {}. Retrying in {:?} (attempt {}/{})",
                            status, delay, retries, max_retries
                        );

                        sleep(delay).await;
                        continue;
                    }

                    return self.handle_response(response).await;
                }
                Err(e) if retries < max_retries => {
                    retries += 1;
                    let delay = self.calculate_backoff_delay(retries);

                    warn!(
                        "Request failed: {}. Retrying in {:?} (attempt {}/{})",
                        e, delay, retries, max_retries
                    );

                    sleep(delay).await;
                    continue;
                }
                Err(e) => {
                    error!("Request failed after {} retries: {}", max_retries, e);
                    return Err(LighterError::Http(Box::new(e)));
                }
            }
        }
    }

    fn calculate_backoff_delay(&self, retry_count: u32) -> Duration {
        // Exponential backoff: 100ms, 200ms, 400ms, 800ms, 1600ms...
        let base_delay_ms = 100;
        let max_delay_ms = 10000; // Cap at 10 seconds

        let delay_ms = std::cmp::min(base_delay_ms * 2_u64.pow(retry_count - 1), max_delay_ms);

        // Add jitter (±25%) to prevent thundering herd
        let jitter = (delay_ms as f64 * 0.25 * rand::random::<f64>()) as u64;
        let final_delay = if rand::random::<bool>() {
            delay_ms + jitter
        } else {
            delay_ms.saturating_sub(jitter)
        };

        Duration::from_millis(final_delay)
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| LighterError::Http(Box::new(e)))?;

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
                        .unwrap_or(body);

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
