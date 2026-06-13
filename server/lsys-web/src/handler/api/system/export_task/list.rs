use crate::common::{
    JsonData, JsonResponse, JsonResult, PageParam, RequestDao, ToOffsetPageParam, UserAuthQueryDao,
};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminExportTaskManage;
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

/// 系统导出任务列表（app_id=0，不按用户过滤）
pub async fn admin_export_list(
    param: &AdminExportListParam,
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

    let page = param.page.to_offset_page_param();
    let export_type_ref = param.export_type.as_deref();

    let tasks = web_dao
        .web_export.export_task
        .list_tasks(
            None,
            Some(0),
            export_type_ref,
            None,
            param.status,
            &page,
        )
        .await?;

    let total = if param.count_num.unwrap_or(false) {
        Some(
            web_dao
                .web_export.export_task
                .count_tasks(None, Some(0), export_type_ref, None, param.status)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(JsonPageData::total(
        tasks, total,
    ))))
}
