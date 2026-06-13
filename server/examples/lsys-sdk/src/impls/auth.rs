use serde::{Deserialize, Serialize};

use crate::client::ServiceClient;
use crate::result::ServiceResult;
use crate::types::ForwardedRequest;

/// Auth verify request parameters
#[derive(Debug, Clone, Serialize)]
pub struct AuthVerifyParam {
    /// Optional: specific permissions to check
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_keys: Option<Vec<String>>,
}

/// Auth verify response
#[derive(Debug, Clone, Deserialize)]
pub struct AuthVerifyResponse {
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub user_id: u64,
    #[serde(deserialize_with = "crate::utils::deserialize_u64_from_string")]
    pub app_id: u64,
    pub nickname: Option<String>,
    pub username: Option<String>,
    pub userdata: Option<String>,
}

impl ServiceClient {
    /// Verify the auth token and get user information
    ///
    /// This method validates the opaque auth token from the forwarded Authorization header
    /// and returns the authenticated user's information.
    ///
    /// # Arguments
    /// * `forward` - Forwarded request information containing the auth token
    /// * `param` - Optional parameters for the verify request
    ///
    /// # Returns
    /// * `Ok(AuthVerifyResponse)` - User information if authentication succeeds
    /// * `Err(ServiceError)` - Error if authentication fails
    pub async fn auth_verify(
        &self,
        forward: ForwardedRequest,
        param: Option<&AuthVerifyParam>,
    ) -> ServiceResult<AuthVerifyResponse> {
        let mut req = self.post("/service/auth/verify")?.forward(forward);

        if let Some(p) = param {
            req = req.json(p);
        }

        req.send_json().await
    }
}
