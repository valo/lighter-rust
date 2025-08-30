use crate::client::SignerClient;
use crate::error::Result;
use crate::models::ApiResponse;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub hash: String,
    pub block_number: u64,
    pub block_hash: String,
    pub transaction_index: u32,
    pub from_address: String,
    pub to_address: Option<String>,
    pub value: String,
    pub gas_used: String,
    pub gas_price: String,
    pub status: TransactionStatus,
    pub timestamp: DateTime<Utc>,
    pub confirmations: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Pending,
    Confirmed,
    Failed,
    Reverted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: u64,
    pub hash: String,
    pub parent_hash: String,
    pub timestamp: DateTime<Utc>,
    pub transaction_count: u32,
    pub gas_used: String,
    pub gas_limit: String,
    pub miner: String,
}

#[derive(Debug)]
pub struct TransactionApi {
    client: SignerClient,
}

impl TransactionApi {
    pub fn new(client: SignerClient) -> Self {
        Self { client }
    }

    pub async fn get_transaction(&self, tx_hash: &str) -> Result<Transaction> {
        let response: ApiResponse<Transaction> = self
            .client
            .api_client()
            .get(&format!("/transactions/{}", tx_hash))
            .await?;

        match response.data {
            Some(transaction) => Ok(transaction),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response
                    .error
                    .unwrap_or_else(|| "Transaction not found".to_string()),
            }),
        }
    }

    pub async fn get_transactions(
        &self,
        address: &str,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> Result<Vec<Transaction>> {
        let mut query_params = vec![format!("address={}", address)];

        if let Some(page) = page {
            query_params.push(format!("page={}", page));
        }
        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }

        let endpoint = format!("/transactions?{}", query_params.join("&"));

        let response: ApiResponse<Vec<Transaction>> =
            self.client.api_client().get(&endpoint).await?;

        match response.data {
            Some(transactions) => Ok(transactions),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to fetch transactions".to_string()),
            }),
        }
    }

    pub async fn get_block(&self, block_number: u64) -> Result<Block> {
        let response: ApiResponse<Block> = self
            .client
            .api_client()
            .get(&format!("/blocks/{}", block_number))
            .await?;

        match response.data {
            Some(block) => Ok(block),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response
                    .error
                    .unwrap_or_else(|| "Block not found".to_string()),
            }),
        }
    }

    pub async fn get_latest_block(&self) -> Result<Block> {
        let response: ApiResponse<Block> = self.client.api_client().get("/blocks/latest").await?;

        match response.data {
            Some(block) => Ok(block),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to fetch latest block".to_string()),
            }),
        }
    }

    pub async fn wait_for_confirmation(
        &self,
        tx_hash: &str,
        required_confirmations: u32,
    ) -> Result<Transaction> {
        let mut attempts = 0;
        let max_attempts = 60; // 5 minutes with 5-second intervals

        loop {
            if attempts >= max_attempts {
                return Err(crate::error::LighterError::Unknown(format!(
                    "Transaction {} not confirmed after {} attempts",
                    tx_hash, max_attempts
                )));
            }

            match self.get_transaction(tx_hash).await {
                Ok(tx) => {
                    if tx.confirmations >= required_confirmations {
                        return Ok(tx);
                    }
                }
                Err(e) => {
                    if attempts == 0 {
                        return Err(e);
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            attempts += 1;
        }
    }
}
