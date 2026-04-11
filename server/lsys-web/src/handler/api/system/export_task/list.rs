use crate::common::{
    JsonData, JsonResponse, JsonResult, PageParam, ToOffsetPageParam, UserAuthQueryDao,
};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use crate::dao::export_task::ExportTaskListAttr;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::JsonPageData;
use serde::Deserialize;

/// 系统导出任务列表参数
#[derive(Debug, Deserialize)]
pub struct AdminExportListParam {
    /// 可选：按 export_type 过滤
    #[serde(default)]
    pub export_type: Option<String>,
    /// 可选：按状态过滤（1=Pending 2=Running 3=Success 4=Failed）
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    /// 分页参数
    pub page: Option<PageParam>,
    /// 是否返回总数
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// 系统导出任务列表（app_id=0，游标分页）
pub async fn admin_export_list(
    param: &AdminExportListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let page = param.page.to_offset_page_param();
    let export_type_ref = param.export_type.as_deref();
    let request_id_ref = req_dao.req_env.request_id.as_deref();

    let tasks = req_dao
        .web_dao
        .web_files
        .export_task
        .list_tasks(
            0,
            Some(0),
            export_type_ref,
            request_id_ref,
            param.status,
            &page,
            &ExportTaskListAttr {
                attr_file: Some(true),
            },
        )
        .await?;

    let total = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_files
                .export_task
                .count_tasks(0, Some(0), export_type_ref, request_id_ref, param.status)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(JsonPageData::total(
        tasks, total,
    ))))
}
