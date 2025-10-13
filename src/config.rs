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
        ensure_trailing_slash(&mut self.base_url);
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
        ensure_trailing_slash(&mut base_url);
        Self {
            ws_url: Url::parse("wss://ws.lighter.xyz").unwrap(),
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
        url.set_path("/");
        return;
    }

    if !path.ends_with('/') {
        path.push('/');
        url.set_path(&path);
    }
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
    }

    #[test]
    fn with_base_url_preserves_path_segments() {
        let config = Config::new()
            .with_base_url("https://example.com/api/v2")
            .expect("valid url");

        assert!(config.base_url.path().ends_with('/'));
        assert_eq!(config.base_url.as_str(), "https://example.com/api/v2/");
    }
}
