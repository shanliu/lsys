use serde::{Deserialize, Serialize};

use crate::client::ServiceClient;
use crate::result::ServiceResult;

/// App feature check request parameters
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AppFeatureParam {
    /// Application ID
    pub app_id: u64,
    /// Feature keys to check
    pub feature_keys: Vec<String>,
}

/// App feature check response
#[derive(Debug, Clone, Deserialize)]
pub struct AppFeatureResponse {
    /// Whether all features are enabled
    #[serde(deserialize_with = "crate::utils::deserialize_bool_from_string")]
    pub enabled: bool,
    /// App owner's user ID
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub app_user_id: u64,
    /// Keys that were denied (if any)
    #[serde(default)]
    pub denied_keys: Vec<String>,
}

/// App secret request parameters
#[derive(Debug, Clone, Serialize)]
pub(crate) struct AppSecretParam {
    /// Application client_id
    pub client_id: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppSecretRecord {
    pub secret_data: serde_json::Value,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub time_out: u64,
}

/// App secret response
#[derive(Debug, Clone, Deserialize)]
pub struct AppSecretResponse {
    /// Application ID
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub app_id: u64,
    /// App owner's user ID
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub user_id: u64,
    /// List of enabled secrets
    pub secrets: Vec<AppSecretRecord>,
}

impl ServiceClient {
    /// Check if an application has specific features enabled
    ///
    /// This method verifies whether the specified application has the given
    /// feature keys enabled. No user authentication required.
    ///
    /// # Arguments
    /// * `app_id` - Application ID to check
    /// * `feature_keys` - Feature keys to check
    ///
    /// # Returns
    /// * `Ok(AppFeatureResponse)` - Feature status, app owner info, and denied keys
    /// * `Err(ServiceError)` - Error if check fails
    pub async fn app_feature_check(
        &self,
        app_id: u64,
        feature_keys: &[&str],
    ) -> ServiceResult<AppFeatureResponse> {
        let param = AppFeatureParam {
            app_id,
            feature_keys: feature_keys.iter().map(|s| s.to_string()).collect(),
        };

        self.post("/service/app/feature")?
            .json(&param)
            .send_json()
            .await
    }

    /// Get application secrets by client_id
    ///
    /// This method retrieves app secrets for REST signature verification.
    ///
    /// # Arguments
    /// * `client_id` - Application client ID
    ///
    /// # Returns
    /// * `Ok(AppSecretResponse)` - App info and secrets
    /// * `Err(ServiceError)` - Error if app not found or disabled
    pub async fn app_secret(&self, client_id: &str) -> ServiceResult<AppSecretResponse> {
        let param = AppSecretParam {
            client_id: client_id.to_string(),
        };

        self.post("/service/app/secret")?
            .json(&param)
            .send_json()
            .await
    }
}
