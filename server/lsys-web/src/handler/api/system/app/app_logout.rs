use crate::common::{JsonResponse, JsonResult};
use crate::common::{RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminApp;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppLogoutParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
}
pub async fn app_logout(
    param: &AppLogoutParam,
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
            &CheckAdminApp {},
        )
        .await?;
    let app = web_dao.web_app.app_dao.app.find_by_id(param.app_id).await?;
    web_dao
        .web_access
        .access_dao
        .auth
        .clear_app_login(app.id)
        .await?;
    Ok(JsonResponse::default())
}
