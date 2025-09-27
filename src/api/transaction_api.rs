use crate::client::ApiClient;
use crate::error::{LighterError, Result};
use crate::signers::FFISigner;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize)]
struct SendTxRequest {
    tx_type: i32,
    tx_info: String,
}

#[derive(Debug, Deserialize)]
pub struct TxResponse {
    pub code: i32,
    pub tx_hash: Option<String>,
    pub message: Option<String>,
}

pub struct LighterTransactionApi {
    client: ApiClient,
    signer: FFISigner,
}

impl LighterTransactionApi {
    pub fn new(client: ApiClient, url: &str, private_key: &str, api_key_index: i32, account_index: i32) -> Result<Self> {
        let signer = FFISigner::new(url, private_key, api_key_index, account_index)?;
        Ok(Self { client, signer })
    }

    async fn send_tx(&self, tx_type: i32, tx_info: String) -> Result<TxResponse> {
        let request = SendTxRequest { tx_type, tx_info };

        let response: TxResponse = self
            .client
            .post("/send_tx", Some(request))
            .await?;

        if response.code != 200 {
            return Err(LighterError::Api {
                status: response.code as u16,
                message: response.message.unwrap_or("Transaction failed".to_string()),
            });
        }

        Ok(response)
    }

    pub async fn create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        _order_type: i32,
        _time_in_force: i32,
        reduce_only: bool,
        trigger_price: i32,
        order_expiry: i64,
        nonce: i64,
    ) -> Result<(serde_json::Value, TxResponse)> {
        let tx_info = self.signer.sign_create_order(
            market_index,
            client_order_index,
            base_amount,
            price,
            is_ask,
            crate::models::common::OrderType::Limit, // Convert from int later
            crate::models::order::TimeInForce::Gtc,  // Convert from int later
            reduce_only,
            trigger_price,
            order_expiry,
            nonce,
        )?;

        let order_data: serde_json::Value = serde_json::from_str(&tx_info)?;
        let response = self.send_tx(1, tx_info).await?; // TX_TYPE_CREATE_ORDER = 1

        Ok((order_data, response))
    }

    pub async fn cancel_order(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        order_id_to_cancel: &str,
        nonce: i64,
    ) -> Result<TxResponse> {
        let tx_info = self.signer.sign_cancel_order(
            market_index,
            client_cancel_index,
            order_id_to_cancel,
            nonce,
        )?;

        self.send_tx(2, tx_info).await // TX_TYPE_CANCEL_ORDER = 2
    }

    pub async fn cancel_all_orders(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        nonce: i64,
    ) -> Result<TxResponse> {
        let tx_info = self.signer.sign_cancel_all_orders(
            market_index,
            client_cancel_index,
            nonce,
        )?;

        self.send_tx(3, tx_info).await // TX_TYPE_CANCEL_ALL = 3
    }

    pub async fn transfer(
        &self,
        receiver: &str,
        amount: i64,
        nonce: i64,
    ) -> Result<TxResponse> {
        let tx_info = self.signer.sign_transfer(receiver, amount, nonce)?;
        self.send_tx(4, tx_info).await // TX_TYPE_TRANSFER = 4
    }

    pub async fn withdraw(
        &self,
        receiver: &str,
        amount: i64,
        nonce: i64,
    ) -> Result<TxResponse> {
        let tx_info = self.signer.sign_withdraw(receiver, amount, nonce)?;
        self.send_tx(5, tx_info).await // TX_TYPE_WITHDRAW = 5
    }
}