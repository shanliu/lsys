//! 服务间应用接口
use crate::common::{JsonData, JsonResponse, JsonResult};
use crate::dao::WebDao;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 应用功能检查参数
#[derive(Debug, Deserialize)]
pub struct AppFeatureParam {
    pub app_id: u64,
    pub feature_keys: Vec<String>,
}

/// 应用功能检查结果
#[derive(Debug, Serialize)]
pub struct AppFeatureResult {
    pub enabled: bool,
    pub app_user_id: u64,
    pub denied_keys: Vec<String>,
}

/// 检查应用是否启用了指定功能
///
/// 该接口用于服务间调用检查某个应用是否启用了特定的功能，
/// 例如SMS、邮件、RBAC等功能
pub async fn feature(param: &AppFeatureParam, web_dao: &WebDao) -> JsonResult<JsonResponse> {
    // 查找应用
    let app = web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(param.app_id)
        .await?;

    // 检查应用状态
    app.app_status_check()?;

    // 检查功能 - 将 Vec<String> 转换为 Vec<&str>
    let keys: Vec<&str> = param.feature_keys.iter().map(|s| s.as_str()).collect();
    let feature_result = web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &keys)
        .await;

    // 收集被拒绝的功能键
    let (enabled, denied_keys) = match feature_result {
        Ok(_) => (true, vec![]),
        Err(e) => {
            // 如果功能检查失败，返回所有请求的功能键作为被拒绝的键
            let denied = param.feature_keys.clone();
            tracing::debug!("Feature check failed: {:?}, denied: {:?}", e, denied);
            (false, denied)
        }
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "enabled": enabled,
        "app_user_id": app.user_id,
        "denied_keys": denied_keys,
    }))))
}

/// 获取应用密钥参数
#[derive(Debug, Deserialize)]
pub struct AppSecretParam {
    pub client_id: String,
}

/// 应用密钥结果
#[derive(Debug, Serialize)]
pub struct AppSecretResult {
    pub app_id: u64,
    pub user_id: u64,
    pub secret_data: serde_json::Value,
}

/// 根据client_id获取应用密钥
///
/// 该接口用于服务间调用获取应用的密钥信息，
/// 主要用于REST签名验证
pub async fn secret(param: &AppSecretParam, web_dao: &WebDao) -> JsonResult<JsonResponse> {
    // 根据client_id查找应用
    let app = web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&param.client_id)
        .await?;

    // 检查应用状态
    app.app_status_check()?;

    // 使用缓存方法获取应用密钥
    let secrets = web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_app_secret_by_client_id(&param.client_id)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "app_id": app.id,
        "user_id": app.user_id,
        "secrets": secrets,
    }))))
}
