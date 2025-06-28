use crate::common::UserAuthQueryDao;
use crate::common::{JsonResponse, JsonResult};
use crate::dao::access::api::system::admin::CheckAdminApp;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppLogoutParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
}
pub async fn app_logout(
    param: &AppLogoutParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
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
        .web_access
        .access_dao
        .auth
        .clear_app_login(app.id)
        .await?;
    Ok(JsonResponse::default())
}
