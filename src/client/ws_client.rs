use crate::config::Config;
use crate::error::{LighterError, Result};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

pub type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsRequest {
    pub id: String,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsResponse {
    pub id: Option<String>,
    pub result: Option<Value>,
    pub error: Option<WsError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug)]
pub struct WebSocketClient {
    config: Config,
    stream: Option<WsStream>,
    subscriptions: HashMap<String, String>,
}

impl WebSocketClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            stream: None,
            subscriptions: HashMap::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WebSocket: {}", self.config.ws_url);

        let (ws_stream, _response) = connect_async(&self.config.ws_url.to_string())
            .await
            .map_err(|e| LighterError::WebSocket(Box::new(e)))?;

        info!("WebSocket connected successfully");
        self.stream = Some(ws_stream);
        Ok(())
    }

    pub async fn subscribe(&mut self, channel: &str, params: Option<Value>) -> Result<String> {
        let request_id = uuid::Uuid::new_v4().to_string();
        let request = WsRequest {
            id: request_id.clone(),
            method: "SUBSCRIBE".to_string(),
            params,
        };

        self.send_request(&request).await?;
        self.subscriptions
            .insert(request_id.clone(), channel.to_string());

        debug!("Subscribed to channel: {} with ID: {}", channel, request_id);
        Ok(request_id)
    }

    pub async fn unsubscribe(&mut self, subscription_id: &str) -> Result<()> {
        let request = WsRequest {
            id: uuid::Uuid::new_v4().to_string(),
            method: "UNSUBSCRIBE".to_string(),
            params: Some(serde_json::json!({
                "subscription_id": subscription_id
            })),
        };

        self.send_request(&request).await?;
        self.subscriptions.remove(subscription_id);

        debug!("Unsubscribed from subscription ID: {}", subscription_id);
        Ok(())
    }

    pub async fn send_request(&mut self, request: &WsRequest) -> Result<()> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            LighterError::WebSocket(Box::new(tungstenite::Error::ConnectionClosed))
        })?;

        let message = serde_json::to_string(request).map_err(LighterError::Json)?;

        stream
            .send(Message::Text(message))
            .await
            .map_err(|e| LighterError::WebSocket(Box::new(e)))?;

        debug!("Sent WebSocket request: {}", request.id);
        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<Value>> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            LighterError::WebSocket(Box::new(tungstenite::Error::ConnectionClosed))
        })?;

        match stream.next().await {
            Some(Ok(Message::Text(text))) => {
                debug!("Received WebSocket message: {}", text);
                let value: Value = serde_json::from_str(&text).map_err(LighterError::Json)?;
                Ok(Some(value))
            }
            Some(Ok(Message::Close(_))) => {
                info!("WebSocket connection closed by server");
                self.stream = None;
                Ok(None)
            }
            Some(Ok(Message::Ping(payload))) => {
                debug!("Received ping, sending pong");
                stream
                    .send(Message::Pong(payload))
                    .await
                    .map_err(|e| LighterError::WebSocket(Box::new(e)))?;
                Ok(None)
            }
            Some(Ok(Message::Pong(_))) => {
                debug!("Received pong");
                Ok(None)
            }
            Some(Ok(_)) => {
                warn!("Received unsupported message type");
                Ok(None)
            }
            Some(Err(e)) => {
                error!("WebSocket error: {}", e);
                Err(LighterError::WebSocket(Box::new(e)))
            }
            None => {
                info!("WebSocket stream ended");
                self.stream = None;
                Ok(None)
            }
        }
    }

    pub async fn close(&mut self) -> Result<()> {
        if let Some(stream) = &mut self.stream {
            stream
                .close(None)
                .await
                .map_err(|e| LighterError::WebSocket(Box::new(e)))?;
            info!("WebSocket connection closed");
        }
        self.stream = None;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    pub fn get_subscriptions(&self) -> &HashMap<String, String> {
        &self.subscriptions
    }
}
