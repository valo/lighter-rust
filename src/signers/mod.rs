pub mod ethereum;
pub mod ffi;

use crate::error::{LighterError, Result};
use crate::models::{
    common::{OrderType, Side},
    order::TimeInForce,
    AccountTier,
};
use serde::Serialize;

pub use ethereum::*;
pub use ffi::*;

fn serialize_payload<T: Serialize>(payload: &T) -> Result<String> {
    serde_json::to_string(payload).map_err(|err| LighterError::Signing(err.to_string()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OrderSignatureData<'a> {
    symbol: &'a str,
    side: Side,
    order_type: OrderType,
    quantity: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_order_id: Option<&'a str>,
    time_in_force: TimeInForce,
    #[serde(skip_serializing_if = "Option::is_none")]
    post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reduce_only: Option<bool>,
    nonce: u64,
}

#[derive(Serialize)]
struct AccountTierSignatureData {
    target_tier: AccountTier,
    nonce: u64,
}

pub fn order_signature_message(
    symbol: &str,
    side: Side,
    order_type: OrderType,
    quantity: &str,
    price: Option<&str>,
    stop_price: Option<&str>,
    client_order_id: Option<&str>,
    time_in_force: TimeInForce,
    post_only: Option<bool>,
    reduce_only: Option<bool>,
    nonce: u64,
) -> Result<String> {
    let payload = OrderSignatureData {
        symbol,
        side,
        order_type,
        quantity,
        price,
        stop_price,
        client_order_id,
        time_in_force,
        post_only,
        reduce_only,
        nonce,
    };

    serialize_payload(&payload)
}

pub fn sign_order_payload(
    signer: &dyn Signer,
    symbol: &str,
    side: Side,
    order_type: OrderType,
    quantity: &str,
    price: Option<&str>,
    stop_price: Option<&str>,
    client_order_id: Option<&str>,
    time_in_force: TimeInForce,
    post_only: Option<bool>,
    reduce_only: Option<bool>,
    nonce: u64,
) -> Result<String> {
    let message = order_signature_message(
        symbol,
        side,
        order_type,
        quantity,
        price,
        stop_price,
        client_order_id,
        time_in_force,
        post_only,
        reduce_only,
        nonce,
    )?;

    signer.sign_message(&message)
}

pub fn account_tier_signature_message(target_tier: AccountTier, nonce: u64) -> Result<String> {
    let payload = AccountTierSignatureData { target_tier, nonce };
    serialize_payload(&payload)
}

pub fn sign_account_tier_payload(
    signer: &dyn Signer,
    target_tier: AccountTier,
    nonce: u64,
) -> Result<String> {
    let message = account_tier_signature_message(target_tier, nonce)?;
    signer.sign_message(&message)
}
