use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::admin::CheckAdminApp;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DisableParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
}
//APP 禁用
pub async fn disable(param: &DisableParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(&RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env), &CheckAdminApp {})
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_disable(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
