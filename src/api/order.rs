use crate::client::ApiClient;
use crate::error::Result;
use crate::models::{ApiResponse, Order, OrderFilter, Trade, Pagination};

#[derive(Debug)]
pub struct OrderApi {
    client: ApiClient,
}

impl OrderApi {
    pub fn new(client: ApiClient) -> Self {
        Self { client }
    }

    pub async fn get_order(&self, order_id: &str) -> Result<Order> {
        let response: ApiResponse<Order> = self
            .client
            .get(&format!("/orders/{}", order_id))
            .await?;

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
}