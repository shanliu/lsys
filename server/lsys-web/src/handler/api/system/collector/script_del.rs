use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ScriptDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub script_id: u64,
}

/// POST /api/system/collector/script_del — 删除系统脚本
pub async fn script_del(
    param: &ScriptDeleteParam,
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

    let affected = req_dao
        .web_dao
        .web_files
        .collector
        .script_delete(param.script_id, Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "affected": affected,
    }))))
}
