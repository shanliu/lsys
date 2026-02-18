use serde::{Deserialize, Serialize};

use crate::client::ServiceClient;
use crate::result::ServiceResult;

/// Operation permission check parameters
#[derive(Debug, Clone, Serialize)]
pub struct ResReqAuthParam {
    /// Operation key
    pub op_key: String,
    /// Whether authentication is required
    pub req_auth: bool,
}

/// Resource check parameters
#[derive(Debug, Clone, Serialize)]
pub struct ResCheckParam {
    /// Resource type
    pub res_type: String,
    /// Resource data
    pub res_data: String,
    /// Resource owner user ID
    pub res_user_id: u64,
    /// Operations to check
    pub ops: Vec<ResReqAuthParam>,
}

/// Session role parameters
#[derive(Debug, Clone, Serialize)]
pub struct RoleCheckParam {
    /// Role key
    pub role_key: String,
    /// User ID for this role
    pub user_id: u64,
}

/// Access check parameters
#[derive(Debug, Clone, Serialize)]
pub struct AccessCheckParam {
    /// Session roles
    pub role_key: Vec<RoleCheckParam>,
    /// Resource checks (2D array - outer is OR, inner is AND)
    pub check_res: Vec<Vec<ResCheckParam>>,
}

/// RBAC check request parameters
#[derive(Debug, Clone, Serialize)]
pub struct RbacCheckParam {
    /// User ID to check permissions for
    pub user_id: u64,
    /// Optional token data for logging
    pub token_data: Option<String>,
    /// Access check parameters
    pub access: AccessCheckParam,
}

/// Check item parameters
#[derive(Debug, Clone, Serialize)]
pub struct RbacCheckItem {
    /// Item name for identification
    pub name: String,
    /// Check parameters
    pub check_res: RbacCheckParam,
}

/// RBAC check request parameters
#[derive(Debug, Clone, Serialize)]
pub struct RbacCheckRequest {
    /// Items to check
    pub menu_res: Vec<RbacCheckItem>,
}

/// RBAC check response item
#[derive(Debug, Clone, Deserialize)]
pub struct RbacCheckStatus {
    /// Whether check passed
    #[serde(deserialize_with = "crate::utils::deserialize_bool_from_string")]
    pub status: bool,
    /// Item name
    pub name: String,
}

/// RBAC check response
#[derive(Debug, Clone, Deserialize)]
pub struct RbacCheckResponse {
    /// Check results
    pub result: Vec<RbacCheckStatus>,
}

impl ServiceClient {
    /// Check RBAC permissions for multiple items
    ///
    /// This method verifies permissions for multiple items at once.
    ///
    /// # Arguments
    /// * `param` - RBAC check parameters
    ///
    /// # Returns
    /// * `Ok(RbacCheckResponse)` - Check results for each item
    /// * `Err(ServiceError)` - Error if request fails
    pub async fn rbac_check(&self, param: &RbacCheckRequest) -> ServiceResult<RbacCheckResponse> {
        self.post("/service/rbac/check")?
            .json(param)
            .send_json()
            .await
    }
}
