use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminExportTaskManage;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

/// 系统活跃导出任务数量参数
#[derive(Debug, Deserialize)]
pub struct AdminExportActiveCountParam {
    /// 仅统计指定类型的活跃任务（可选）
    #[serde(default)]
    pub export_type: Option<String>,
}

/// 活跃导出任务数（Pending + Running，系统维度，不按用户过滤）
///
/// 前端初始化时调用一次；返回 > 0 则开始轮询，返回 0 则停止。
pub async fn admin_export_active_count(
    param: &AdminExportActiveCountParam,
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
            &CheckAdminExportTaskManage {},
        )
        .await?;

    let count = web_dao
        .web_export.export_task
        .count_active_tasks(None, Some(0), param.export_type.as_deref())
        .await?;

    Ok(JsonResponse::data(JsonData::body(
        json!({ "count": count }),
    )))
}
