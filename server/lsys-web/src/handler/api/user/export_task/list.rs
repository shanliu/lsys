use crate::common::{
    JsonData, JsonPageData, JsonResponse, JsonResult, ToOffsetPageParam, UserAuthQueryDao,
};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileView;
use crate::dao::export_task::ExportTaskListAttr;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
/// 导出任务列表参数
#[derive(Debug, Deserialize)]
pub struct ExportListParam {
    /// 可选：按 export_type 过滤
    #[serde(default)]
    pub export_type: Option<String>,
    /// 可选：按状态过滤（1=Pending 2=Running 3=Success 4=Failed）
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    /// 分页参数
    pub page: Option<crate::common::PageParam>,
    /// 是否返回总数
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// 导出任务列表（游标分页）
pub async fn user_export_list(
    param: &ExportListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();

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

    let page = param.page.to_offset_page_param();
    let export_type_ref = param.export_type.as_deref();
    let request_id_ref = req_dao.req_env.request_id.as_deref();

    let tasks = req_dao
        .web_dao
        .web_files
        .export_task
        .list_tasks(
            user_id,
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
                .count_tasks(
                    user_id,
                    Some(0),
                    export_type_ref,
                    request_id_ref,
                    param.status,
                )
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(JsonPageData::total(
        tasks, total,
    ))))
}
