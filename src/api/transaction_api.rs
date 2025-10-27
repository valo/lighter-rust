use crate::client::ApiClient;
use crate::error::{LighterError, Result};
use crate::signers::FFISigner;
use serde::{Deserialize, Serialize};
use serde_json;
use tracing::debug;

#[derive(Debug, Clone, Serialize)]
struct SendTxRequest {
    tx_type: i32,
    tx_info: String,
}

const TX_TYPE_CREATE_ORDER: i32 = 14;
const TX_TYPE_CANCEL_ORDER: i32 = 15;
const TX_TYPE_CANCEL_ALL_ORDERS: i32 = 16;
const TX_TYPE_TRANSFER: i32 = 12;
const TX_TYPE_WITHDRAW: i32 = 13;

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
    pub fn new(
        client: ApiClient,
        url: &str,
        private_key: &str,
        api_key_index: i32,
        account_index: i32,
    ) -> Result<Self> {
        let signer = FFISigner::new(url, private_key, api_key_index, account_index)?;
        Ok(Self { client, signer })
    }

    pub fn with_signer(client: ApiClient, signer: FFISigner) -> Self {
        Self { client, signer }
    }

    async fn send_tx(&self, tx_type: i32, tx_info: String) -> Result<TxResponse> {
        let payload = SendTxRequest { tx_type, tx_info };

        if let Ok(payload_json) = serde_json::to_string(&payload) {
            debug!(target: "lighter::http", payload = payload_json, "Sending Lighter HTTP sendTx request");
            debug!(target: "lighter::http", "Lighter HTTP sendTx payload: {}", payload_json);
        }

        let response: TxResponse = self.client.post("/sendTx", Some(payload)).await?;

        if response.code != 200 {
            return Err(LighterError::Api {
                status: response.code as u16,
                message: response.message.unwrap_or("Transaction failed".to_string()),
            });
        }

        Ok(response)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        market_index: i32,
        client_order_index: i64,
        base_amount: i64,
        price: i32,
        is_ask: bool,
        order_type: crate::models::common::OrderType,
        time_in_force: crate::models::order::TimeInForce,
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
            order_type,
            time_in_force,
            reduce_only,
            trigger_price,
            order_expiry,
            nonce,
        )?;

        let order_data: serde_json::Value = serde_json::from_str(&tx_info)?;
        let response = self.send_tx(TX_TYPE_CREATE_ORDER, tx_info).await?;

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

        self.send_tx(TX_TYPE_CANCEL_ORDER, tx_info).await
    }

    pub async fn cancel_all_orders(
        &self,
        market_index: i32,
        client_cancel_index: i64,
        nonce: i64,
    ) -> Result<TxResponse> {
        let tx_info =
            self.signer
                .sign_cancel_all_orders(market_index, client_cancel_index, nonce)?;

        self.send_tx(TX_TYPE_CANCEL_ALL_ORDERS, tx_info).await
    }

    pub async fn transfer(&self, receiver: &str, amount: i64, nonce: i64) -> Result<TxResponse> {
        let tx_info = self.signer.sign_transfer(receiver, amount, nonce)?;
        self.send_tx(TX_TYPE_TRANSFER, tx_info).await
    }

    pub async fn withdraw(&self, receiver: &str, amount: i64, nonce: i64) -> Result<TxResponse> {
        let tx_info = self.signer.sign_withdraw(receiver, amount, nonce)?;
        self.send_tx(TX_TYPE_WITHDRAW, tx_info).await
    }
}
