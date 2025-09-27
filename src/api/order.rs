use crate::client::SignerClient;
use crate::error::Result;
use crate::models::{
    ApiResponse, CancelOrderRequest, CreateOrderRequest, Order, OrderFilter, OrderType, Pagination,
    Side, TimeInForce, Trade,
};
use crate::signers::{sign_cancel_payload, sign_order_payload};

#[derive(Debug)]
pub struct OrderApi {
    client: SignerClient,
}

impl OrderApi {
    pub fn new(client: SignerClient) -> Self {
        Self { client }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        symbol: &str,
        side: Side,
        order_type: OrderType,
        quantity: &str,
        price: Option<&str>,
        client_order_id: Option<&str>,
        time_in_force: Option<TimeInForce>,
        post_only: Option<bool>,
        reduce_only: Option<bool>,
    ) -> Result<Order> {
        let nonce = self.client.generate_nonce()?;

        let side_str = match side {
            Side::Buy => "BUY",
            Side::Sell => "SELL",
        };

        let signature = sign_order_payload(
            self.client.signer().as_ref(),
            symbol,
            side_str,
            quantity,
            price,
            nonce,
        )?;

        let request = CreateOrderRequest {
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity: quantity.to_string(),
            price: price.map(String::from),
            stop_price: None,
            client_order_id: client_order_id.map(String::from),
            time_in_force: time_in_force.unwrap_or(TimeInForce::Gtc),
            post_only,
            reduce_only,
            signature,
            nonce,
        };

        let response: ApiResponse<Order> = self
            .client
            .api_client()
            .post("/orders", Some(request))
            .await?;

        match response.data {
            Some(order) => Ok(order),
            None => Err(crate::error::LighterError::OrderValidation(
                response
                    .error
                    .unwrap_or_else(|| "Failed to create order".to_string()),
            )),
        }
    }

    pub async fn cancel_order(
        &self,
        order_id: Option<&str>,
        client_order_id: Option<&str>,
        symbol: Option<&str>,
    ) -> Result<()> {
        if order_id.is_none() && client_order_id.is_none() {
            return Err(crate::error::LighterError::OrderValidation(
                "Either order_id or client_order_id must be provided".to_string(),
            ));
        }

        let nonce = self.client.generate_nonce()?;

        let signature = sign_cancel_payload(
            self.client.signer().as_ref(),
            order_id,
            client_order_id,
            symbol,
            nonce,
        )?;

        let request = CancelOrderRequest {
            order_id: order_id.map(String::from),
            client_order_id: client_order_id.map(String::from),
            symbol: symbol.map(String::from),
            signature,
            nonce,
        };

        let response: ApiResponse<()> = self
            .client
            .api_client()
            .post("/orders/cancel", Some(request))
            .await?;

        if !response.success {
            return Err(crate::error::LighterError::Api {
                status: 400,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to cancel order".to_string()),
            });
        }

        Ok(())
    }

    pub async fn cancel_all_orders(&self, symbol: Option<&str>) -> Result<u32> {
        let nonce = self.client.generate_nonce()?;

        let signature =
            sign_cancel_payload(self.client.signer().as_ref(), None, None, symbol, nonce)?;

        let request = CancelOrderRequest {
            order_id: None,
            client_order_id: None,
            symbol: symbol.map(String::from),
            signature,
            nonce,
        };

        let response: ApiResponse<serde_json::Value> = self
            .client
            .api_client()
            .post("/orders/cancel-all", Some(request))
            .await?;

        match response.data {
            Some(data) => {
                let cancelled_count = data
                    .get("cancelled_count")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                Ok(cancelled_count)
            }
            None => Err(crate::error::LighterError::Api {
                status: 400,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to cancel all orders".to_string()),
            }),
        }
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order> {
        let response: ApiResponse<Order> = self
            .client
            .api_client()
            .get(&format!("/orders/{}", order_id))
            .await?;

        match response.data {
            Some(order) => Ok(order),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response
                    .error
                    .unwrap_or_else(|| "Order not found".to_string()),
            }),
        }
    }

    pub async fn get_orders(
        &self,
        filter: Option<OrderFilter>,
    ) -> Result<(Vec<Order>, Option<Pagination>)> {
        let mut query_params = Vec::new();

        if let Some(filter) = filter {
            if let Some(symbol) = filter.symbol {
                query_params.push(format!("symbol={}", symbol));
            }
            if let Some(status) = filter.status {
                query_params.push(format!("status={:?}", status));
            }
            if let Some(side) = filter.side {
                query_params.push(format!("side={:?}", side));
            }
            if let Some(order_type) = filter.order_type {
                query_params.push(format!("order_type={:?}", order_type));
            }
            if let Some(page) = filter.page {
                query_params.push(format!("page={}", page));
            }
            if let Some(limit) = filter.limit {
                query_params.push(format!("limit={}", limit));
            }
        }

        let endpoint = if query_params.is_empty() {
            "/orders".to_string()
        } else {
            format!("/orders?{}", query_params.join("&"))
        };

        let response: ApiResponse<serde_json::Value> =
            self.client.api_client().get(&endpoint).await?;

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
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to fetch orders".to_string()),
            }),
        }
    }

    pub async fn get_trades(&self, symbol: Option<&str>) -> Result<Vec<Trade>> {
        let endpoint = match symbol {
            Some(sym) => format!("/trades?symbol={}", sym),
            None => "/trades".to_string(),
        };

        let response: ApiResponse<Vec<Trade>> = self.client.api_client().get(&endpoint).await?;

        match response.data {
            Some(trades) => Ok(trades),
            None => Err(crate::error::LighterError::Api {
                status: 500,
                message: response
                    .error
                    .unwrap_or_else(|| "Failed to fetch trades".to_string()),
            }),
        }
    }
}
