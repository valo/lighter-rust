use crate::client::{ApiClient, SignerClient};
use crate::error::Result;
use crate::models::{
    common::{OrderType, Side},
    order::{CreateOrderRequest, TimeInForce},
    ApiResponse, Order, OrderFilter, Pagination, Trade,
};
use crate::signers::sign_order_payload;
use std::sync::Arc;

#[derive(Debug)]
pub struct OrderApi {
    client: ApiClient,
    signer_client: Arc<SignerClient>,
}

impl OrderApi {
    pub fn new(signer_client: SignerClient) -> Self {
        let client = signer_client.api_client().clone();
        Self {
            client,
            signer_client: Arc::new(signer_client),
        }
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order> {
        let response: ApiResponse<Order> =
            self.client.get(&format!("/orders/{}", order_id)).await?;

        match response.data {
            Some(order) => Ok(order),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response.error.unwrap_or("Not found".to_string()),
            }),
        }
    }

    pub async fn get_orders(
        &self,
        filter: Option<OrderFilter>,
    ) -> Result<(Vec<Order>, Option<Pagination>)> {
        let endpoint = build_orders_endpoint(filter.as_ref());

        let response: ApiResponse<serde_json::Value> = self.client.get(&endpoint).await?;

        match response.data {
            Some(data) => {
                let orders: Vec<Order> = serde_json::from_value(
                    data.get("orders")
                        .cloned()
                        .unwrap_or(serde_json::Value::Array(vec![])),
                )
                .unwrap_or_default();

                let pagination: Option<Pagination> = data
                    .get("pagination")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());

                Ok((orders, pagination))
            }
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response.error.unwrap_or("Failed".to_string()),
            }),
        }
    }

    pub async fn get_trades(&self, symbol: Option<&str>) -> Result<Vec<Trade>> {
        let endpoint = match symbol {
            Some(sym) => format!("/trades?symbol={}", sym),
            None => "/trades".to_string(),
        };

        let response: ApiResponse<Vec<Trade>> = self.client.get(&endpoint).await?;

        match response.data {
            Some(trades) => Ok(trades),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response.error.unwrap_or("Failed".to_string()),
            }),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        quantity: &str,
        price: Option<&str>,
        stop_price: Option<&str>,
        client_order_id: Option<&str>,
        time_in_force: Option<TimeInForce>,
        post_only: Option<bool>,
        reduce_only: Option<bool>,
    ) -> Result<Order> {
        // Get nonce and signature
        let nonce = self.signer_client.generate_nonce()?;

        let price_owned = price.map(str::to_string);
        let stop_price_owned = stop_price.map(str::to_string);
        let client_order_id_owned = client_order_id.map(str::to_string);
        let time_in_force_value = time_in_force.unwrap_or(TimeInForce::Gtc);

        let signature = sign_order_payload(
            self.signer_client.signer().as_ref(),
            symbol,
            side,
            order_type,
            quantity,
            price_owned.as_deref(),
            stop_price_owned.as_deref(),
            client_order_id_owned.as_deref(),
            time_in_force_value,
            post_only,
            reduce_only,
            nonce,
        )?;

        let request = CreateOrderRequest {
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity: quantity.to_string(),
            price: price_owned,
            stop_price: stop_price_owned,
            time_in_force: time_in_force_value,
            client_order_id: client_order_id_owned,
            nonce,
            signature,
            post_only,
            reduce_only,
        };

        let response: ApiResponse<Order> = self
            .signer_client
            .post_signed("/orders", Some(&request))
            .await?;

        match response.data {
            Some(order) => Ok(order),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response
                    .error
                    .unwrap_or("Failed to create order".to_string()),
            }),
        }
    }

    pub async fn cancel_order(
        &self,
        order_id: Option<&str>,
        client_order_id: Option<&str>,
        symbol: Option<&str>,
    ) -> Result<()> {
        let endpoint = match (order_id, client_order_id, symbol) {
            (Some(id), _, _) => format!("/orders/{}", id),
            (None, Some(cid), Some(sym)) => {
                format!("/orders?client_order_id={}&symbol={}", cid, sym)
            }
            _ => {
                return Err(crate::error::LighterError::Api {
                    status: 400,
                    message: "Must provide either order_id or both client_order_id and symbol"
                        .to_string(),
                })
            }
        };

        let response: ApiResponse<serde_json::Value> =
            self.signer_client.delete_signed(&endpoint).await?;

        if response.error.is_some() {
            return Err(crate::error::LighterError::Api {
                status: 500,
                message: response.error.unwrap(),
            });
        }

        Ok(())
    }
}

fn build_orders_endpoint(filter: Option<&OrderFilter>) -> String {
    let mut query_params = Vec::new();

    if let Some(filter) = filter {
        if let Some(symbol) = filter.symbol.as_ref() {
            query_params.push(format!("symbol={}", symbol));
        }
        if let Some(status) = filter.status {
            query_params.push(format!("status={}", status.as_str()));
        }
        if let Some(side) = filter.side {
            query_params.push(format!("side={}", side.as_str()));
        }
        if let Some(order_type) = filter.order_type {
            query_params.push(format!("order_type={}", order_type.as_str()));
        }
        if let Some(start_time) = filter.start_time.as_ref() {
            query_params.push(format!("start_time={}", start_time.timestamp_millis()));
        }
        if let Some(end_time) = filter.end_time.as_ref() {
            query_params.push(format!("end_time={}", end_time.timestamp_millis()));
        }
        if let Some(page) = filter.page {
            query_params.push(format!("page={}", page));
        }
        if let Some(limit) = filter.limit {
            query_params.push(format!("limit={}", limit));
        }
    }

    if query_params.is_empty() {
        "/orders".to_string()
    } else {
        format!("/orders?{}", query_params.join("&"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OrderStatus, OrderType, Side};
    use chrono::{TimeZone, Timelike, Utc};

    #[test]
    fn build_orders_endpoint_without_filters_returns_base_path() {
        assert_eq!(build_orders_endpoint(None), "/orders");
    }

    #[test]
    fn build_orders_endpoint_serializes_filters_in_api_format() {
        let filter = OrderFilter {
            symbol: Some("BTC-USDC".to_string()),
            status: Some(OrderStatus::Open),
            side: Some(Side::Buy),
            order_type: Some(OrderType::Limit),
            start_time: Some(
                Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0)
                    .unwrap()
                    .with_nanosecond(123_000_000)
                    .unwrap(),
            ),
            end_time: Some(
                Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0)
                    .unwrap()
                    .with_nanosecond(456_000_000)
                    .unwrap(),
            ),
            page: Some(2),
            limit: Some(50),
        };

        let endpoint = build_orders_endpoint(Some(&filter));

        assert!(endpoint.starts_with("/orders?"));
        assert!(endpoint.contains("symbol=BTC-USDC"));
        assert!(endpoint.contains("status=OPEN"));
        assert!(endpoint.contains("side=BUY"));
        assert!(endpoint.contains("order_type=LIMIT"));
        assert!(endpoint.contains("start_time=1704067200123"));
        assert!(endpoint.contains("end_time=1704153600456"));
        assert!(endpoint.contains("page=2"));
        assert!(endpoint.contains("limit=50"));
    }
}
