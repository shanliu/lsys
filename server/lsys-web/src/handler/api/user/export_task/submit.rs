use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

// ── 处理函数 ──────────────────────────────────────────────────────────────────
/// 提交导出任务参数
#[derive(Debug, Deserialize)]
pub struct ExportSubmitParam {
    /// 导出类型标识（需已注册到 WebExportTask）
    pub export_type: String,
    /// 导出过滤参数（各 Exporter 自行解析）
    #[serde(default)]
    pub params: Option<serde_json::Value>,
}
/// 提交导出任务
pub async fn user_export_submit(
    param: &ExportSubmitParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();
    // app_id=0 → 用户本身，无需校验 app；app_id>0 → 用户应用，校验 app 并取 app.user_id

    req_dao
        .web_dao
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

    let task_id = req_dao
        .web_dao
        .web_files
        .export_task
        .submit(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            0,
            0,
            user_id,
            &param.export_type,
            params,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({ "id": task_id }))))
}
