use crate::client::SignerClient;
use crate::error::Result;
use crate::models::{Account, AccountStats, AccountTier, AccountTierSwitchRequest, ApiResponse};
use crate::signers::sign_account_tier_payload;
use tracing::{debug, info, instrument, warn};

#[derive(Debug)]
pub struct AccountApi {
    client: SignerClient,
}

impl AccountApi {
    pub fn new(client: SignerClient) -> Self {
        Self { client }
    }

    #[instrument(skip(self))]
    pub async fn get_account(&self) -> Result<Account> {
        debug!("Fetching account information");

        let response: ApiResponse<Account> = self.client.api_client().get("/account").await?;

        match response.data {
            Some(account) => {
                info!("Successfully retrieved account: {}", account.id);
                debug!(
                    "Account tier: {:?}, Balances: {}",
                    account.tier,
                    account.balances.len()
                );
                Ok(account)
            }
            None => {
                warn!("Account not found in response");
                Err(crate::error::LighterError::Api {
                    status: 404,
                    message: response
                        .error
                        .unwrap_or_else(|| "Account not found".to_string()),
                })
            }
        }
    }

    pub async fn get_account_stats(&self) -> Result<AccountStats> {
        let response: ApiResponse<AccountStats> =
            self.client.api_client().get("/account/stats").await?;

        match response.data {
            Some(stats) => Ok(stats),
            None => Err(crate::error::LighterError::Api {
                status: 404,
                message: response
                    .error
                    .unwrap_or_else(|| "Account stats not found".to_string()),
            }),
        }
    }

    pub async fn change_account_tier(&self, target_tier: AccountTier) -> Result<()> {
        let nonce = self.client.generate_nonce()?;

        let signature =
            sign_account_tier_payload(self.client.signer().as_ref(), target_tier, nonce)?;

        let request = AccountTierSwitchRequest {
            target_tier,
            signature,
            nonce,
        };

        let response: ApiResponse<()> = self
            .client
            .api_client()
            .post("/account/change-tier", Some(request))
            .await?;

        if !response.success {
            return Err(crate::error::LighterError::AccountTierSwitch(
                response
                    .error
                    .unwrap_or_else(|| "Failed to change account tier".to_string()),
            ));
        }

        Ok(())
    }

    pub async fn can_switch_tier(&self) -> Result<bool> {
        let account = self.get_account().await?;

        let has_positions = !account.positions.is_empty();
        let has_open_orders = false; // TODO: Check for open orders

        if has_positions || has_open_orders {
            return Ok(false);
        }

        if let Some(allowed_at) = account.tier_switch_allowed_at {
            let now = chrono::Utc::now();
            Ok(now >= allowed_at)
        } else {
            Ok(true)
        }
    }

    pub async fn get_balances(&self) -> Result<Vec<crate::models::Balance>> {
        let account = self.get_account().await?;
        Ok(account.balances)
    }

    pub async fn get_positions(&self) -> Result<Vec<crate::models::Position>> {
        let account = self.get_account().await?;
        Ok(account.positions)
    }
}
