use crate::common::JsonResult;

use crate::common::{JsonData, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppEdit;
use lsys_access::dao::AccessSession;

pub struct AppRequestExterLoginFeatureData {
    pub app_id: u64,
}

pub async fn app_request_inner_feature_exter_login_request(
    param: &AppRequestExterLoginFeatureData,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_exter_login_request(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

pub struct AppRequestExterSubAppData {
    pub app_id: u64,
}

pub async fn app_request_inner_feature_sub_app_request(
    param: &AppRequestExterSubAppData,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_request(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}
