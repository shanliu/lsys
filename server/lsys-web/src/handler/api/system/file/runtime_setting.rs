use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

// ==================== 获取运行时配置 ====================

/// 管理员获取文件运行时配置
pub async fn admin_runtime_setting_get(
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let runtime_setting = web_dao.web_file.file_dao.runtime_setting();
    let config = runtime_setting.get_config().await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "local_public_url_prefix": config.local_public_url_prefix,
        "max_download_concurrency": config.max_download_concurrency,
        "download_timeout_secs": config.download_timeout_secs,
        "upload_max_file_size": config.upload_max_file_size,
    }))))
}

// ==================== 更新运行时配置 ====================

#[derive(Debug, Deserialize)]
pub struct AdminRuntimeSettingUpdateParam {
    pub local_public_url_prefix: String,
    pub max_download_concurrency: usize,
    pub download_timeout_secs: u64,
    #[serde(default)]
    pub upload_max_file_size: Option<u64>,
}

/// 管理员更新文件运行时配置
pub async fn admin_runtime_setting_update(
    param: &AdminRuntimeSettingUpdateParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let runtime_setting = web_dao.web_file.file_dao.runtime_setting();
    let old_config = runtime_setting.get_config().await?;

    let upload_max_file_size = param
        .upload_max_file_size
        .unwrap_or(old_config.upload_max_file_size);

    let config = lsys_file::dao::FileRuntimeSettingData {
        local_public_url_prefix: param.local_public_url_prefix.clone(),
        max_download_concurrency: param.max_download_concurrency,
        download_timeout_secs: param.download_timeout_secs,
        upload_max_file_size,
    };

    runtime_setting
        .update_config(&config, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;

    if old_config.local_public_url_prefix != config.local_public_url_prefix {
        web_dao.web_file.file_dao.clear_all_file_url_cache().await;
    }

    Ok(JsonResponse::data(JsonData::body(json!({
        "message": "Runtime settings updated successfully"
    }))))
}
