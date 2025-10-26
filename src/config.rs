use crate::error::{LighterError, Result};
use url::Url;

#[derive(Debug, Clone)]
pub struct Config {
    pub base_url: Url,
    pub ws_url: Url,
    pub api_key: Option<String>,
    pub timeout_secs: u64,
    pub max_retries: u32,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_api_key<S: Into<String>>(mut self, api_key: S) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn with_base_url<S: AsRef<str>>(mut self, url: S) -> Result<Self> {
        self.base_url = Url::parse(url.as_ref())
            .map_err(|e| LighterError::Config(format!("Invalid base URL: {}", e)))?;
        ensure_api_path(&mut self.base_url);
        ensure_trailing_slash(&mut self.base_url);
        self.ws_url = derive_ws_url(&self.base_url)?;
        Ok(self)
    }

    pub fn with_ws_url<S: AsRef<str>>(mut self, url: S) -> Result<Self> {
        self.ws_url = Url::parse(url.as_ref())
            .map_err(|e| LighterError::Config(format!("Invalid WebSocket URL: {}", e)))?;
        Ok(self)
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut base_url = Url::parse("https://mainnet.zklighter.elliot.ai/api/v1/").unwrap();
        ensure_api_path(&mut base_url);
        ensure_trailing_slash(&mut base_url);
        let ws_url = derive_ws_url(&base_url).unwrap();
        Self {
            ws_url,
            api_key: None,
            timeout_secs: 30,
            max_retries: 3,
            base_url,
        }
    }
}

fn ensure_trailing_slash(url: &mut Url) {
    let mut path = url.path().to_string();

    if path.is_empty() {
        path.push('/');
    }

    if !path.ends_with('/') {
        path.push('/');
    }

    if path != url.path() {
        url.set_path(&path);
    }
}

fn ensure_api_path(url: &mut Url) {
    let path = url.path().trim_matches('/');
    if path.is_empty() {
        url.set_path("/api/v1/");
    }
}

fn derive_ws_url(base_url: &Url) -> Result<Url> {
    let scheme = match base_url.scheme() {
        "https" => "wss",
        "http" => "ws",
        other => {
            return Err(LighterError::Config(format!(
                "Unsupported base URL scheme: {}",
                other
            )))
        }
    };

    let host = base_url
        .host_str()
        .ok_or_else(|| LighterError::Config("Base URL missing host".to_string()))?;

    let mut ws = Url::parse(&format!("{}://{}", scheme, host))
        .map_err(|e| LighterError::Config(format!("invalid websocket host: {}", e)))?;

    if let Some(port) = base_url.port() {
        ws.set_port(Some(port))
            .map_err(|_| LighterError::Config("Failed to set websocket port".to_string()))?;
    }

    ws.set_path("/stream");
    Ok(ws)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_base_url_has_trailing_slash() {
        let config = Config::default();
        assert!(config.base_url.path().ends_with('/'));
        assert_eq!(
            config.base_url.as_str(),
            "https://mainnet.zklighter.elliot.ai/api/v1/"
        );
        assert_eq!(
            config.ws_url.as_str(),
            "wss://mainnet.zklighter.elliot.ai/stream"
        );
    }

    #[test]
    fn with_base_url_preserves_path_segments() {
        let config = Config::new()
            .with_base_url("https://example.com/api/v2")
            .expect("valid url");

        assert!(config.base_url.path().ends_with('/'));
        assert_eq!(config.base_url.as_str(), "https://example.com/api/v2/");
    }

    #[test]
    fn with_base_url_without_path_defaults_to_api_v1() {
        let config = Config::new()
            .with_base_url("https://example.com")
            .expect("valid url");

        assert_eq!(
            config.base_url.as_str(),
            "https://example.com/api/v1/"
        );
        assert_eq!(config.ws_url.as_str(), "wss://example.com/stream");
    }
}
