use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
/// 提交导出任务参数
#[derive(Debug, Deserialize)]
pub struct ExportSubmitParam {
    /// 应用 ID
    #[serde(default, deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    /// 导出类型标识（需已注册到 WebExportTask）
    pub export_type: String,
    /// 导出过滤参数（各 Exporter 自行解析）
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}

/// 提交导出任务
pub async fn app_export_submit(
    param: &ExportSubmitParam,
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
    let user_id = auth_data.user_id();
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao, web_dao).await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileView {
                res_user_id: user_id,
            },
        )
        .await?;

    let params = param
        .params
        .clone()
        .unwrap_or_else(|| serde_json::Value::Object(Default::default()));

    let task_id = web_dao
        .web_export.export_task
        .submit(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            app.id,
            app.user_id,
            user_id,
            &param.export_type,
            params,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({ "id": task_id }))))
}
