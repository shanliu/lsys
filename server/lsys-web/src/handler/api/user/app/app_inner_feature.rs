use crate::common::JsonResult;

use crate::common::{JsonResponse, UserAuthQueryDao};
use crate::dao::access::api::system::user::CheckUserAppEdit;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestExterLoginFeatureParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
}

pub async fn request_inner_feature_exter_login_request(
    param: &RequestExterLoginFeatureParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
        )
        .await?;
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .exter_login
        .inner_feature_exter_login_request(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
