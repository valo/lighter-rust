use crate::client::SignerClient;
use crate::error::Result;
use crate::models::ApiResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    pub symbol: String,
    pub interval: String,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub quote_volume: String,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CandlestickInterval {
    #[serde(rename = "1m")]
    OneMinute,
    #[serde(rename = "5m")]
    FiveMinutes,
    #[serde(rename = "15m")]
    FifteenMinutes,
    #[serde(rename = "30m")]
    ThirtyMinutes,
    #[serde(rename = "1h")]
    OneHour,
    #[serde(rename = "4h")]
    FourHours,
    #[serde(rename = "1d")]
    OneDay,
    #[serde(rename = "1w")]
    OneWeek,
}

impl CandlestickInterval {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::OneMinute => "1m",
            Self::FiveMinutes => "5m",
            Self::FifteenMinutes => "15m",
            Self::ThirtyMinutes => "30m",
            Self::OneHour => "1h",
            Self::FourHours => "4h",
            Self::OneDay => "1d",
            Self::OneWeek => "1w",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketStats {
    pub symbol: String,
    pub price_change: String,
    pub price_change_percent: String,
    pub last_price: String,
    pub bid_price: String,
    pub ask_price: String,
    pub volume: String,
    pub quote_volume: String,
    pub high_price: String,
    pub low_price: String,
    pub open_price: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    pub symbol: String,
    pub price: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CandlestickApi {
    client: SignerClient,
}

impl CandlestickApi {
    pub fn new(client: SignerClient) -> Self {
        Self { client }
    }

    pub async fn get_candlesticks(
        &self,
        symbol: &str,
        interval: CandlestickInterval,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> Result<Vec<Candlestick>> {
        let mut query_params = vec![
            format!("symbol={}", symbol),
            format!("interval={}", interval.as_str()),
        ];

        if let Some(start) = start_time {
            query_params.push(format!("start_time={}", start.timestamp_millis()));
        }
        if let Some(end) = end_time {
            query_params.push(format!("end_time={}", end.timestamp_millis()));
        }
        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }

        let endpoint = format!("/candlesticks?{}", query_params.join("&"));

        let response: ApiResponse<Vec<Candlestick>> =
            self.client.api_client().get(&endpoint).await?;

        match response.data {
            Some(candlesticks) => Ok(candlesticks),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to fetch candlesticks".to_string()),
            }),
        }
    }

    pub async fn get_market_stats(&self, symbol: &str) -> Result<MarketStats> {
        let response: ApiResponse<MarketStats> = self
            .client
            .api_client()
            .get(&format!("/market/stats/{}", symbol))
            .await?;

        match response.data {
            Some(stats) => Ok(stats),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response
                    .error
                    .unwrap_or_else(|| "Market stats not found".to_string()),
            }),
        }
    }

    pub async fn get_all_market_stats(&self) -> Result<Vec<MarketStats>> {
        let response: ApiResponse<Vec<MarketStats>> =
            self.client.api_client().get("/market/stats").await?;

        match response.data {
            Some(stats) => Ok(stats),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to fetch market stats".to_string()),
            }),
        }
    }

    pub async fn get_ticker(&self, symbol: &str) -> Result<Ticker> {
        let response: ApiResponse<Ticker> = self
            .client
            .api_client()
            .get(&format!("/ticker/{}", symbol))
            .await?;

        match response.data {
            Some(ticker) => Ok(ticker),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response
                    .error
                    .unwrap_or_else(|| "Ticker not found".to_string()),
            }),
        }
    }

    pub async fn get_all_tickers(&self) -> Result<Vec<Ticker>> {
        let response: ApiResponse<Vec<Ticker>> = self.client.api_client().get("/ticker").await?;

        match response.data {
            Some(tickers) => Ok(tickers),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to fetch tickers".to_string()),
            }),
        }
    }

    pub async fn get_order_book(
        &self,
        symbol: &str,
        depth: Option<u32>,
    ) -> Result<crate::models::OrderBook> {
        let mut endpoint = format!("/orderbook/{}", symbol);

        if let Some(depth) = depth {
            endpoint = format!("{}?depth={}", endpoint, depth);
        }

        let response: ApiResponse<crate::models::OrderBook> =
            self.client.api_client().get(&endpoint).await?;

        match response.data {
            Some(orderbook) => Ok(orderbook),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response
                    .error
                    .unwrap_or_else(|| "Order book not found".to_string()),
            }),
        }
    }
}
