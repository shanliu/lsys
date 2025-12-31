use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult};
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{LimitParam, UserAuthQueryDao},
    dao::access::api::system::admin::CheckAdminUserManage,
};
use lsys_access::dao::{AccessError, AccessSession, SessionDataParam};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct LoginHistoryParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub oauth_app_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub is_enable: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub limit: Option<LimitParam>,
}
pub async fn login_history(
    param: &LoginHistoryParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminUserManage {},
        )
        .await?;
    let session_param = SessionDataParam {
        app_id: Some(param.app_id),
        oauth_app_id: param.oauth_app_id,
        user_id: param.user_id,
        is_enable: param.is_enable,
    };
    let (res, next) = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .session_data(
            &session_param,
            param.limit.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .session_count(&session_param)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(req_dao, res, user_id,false) ,
        "next": next,
        "total":count,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct UserLogoutParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub oauth_app_id: u64,
    pub token_data: String,
}
pub async fn user_logout(
    param: &UserLogoutParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminUserManage {},
        )
        .await?;
    let session_res = req_dao
        .web_dao
        .web_access
        .access_dao
        .auth
        .login_data(param.app_id, param.oauth_app_id, &param.token_data)
        .await;
    match session_res {
        Ok(sess) => {
            req_dao
                .web_dao
                .web_access
                .access_dao
                .auth
                .do_logout(&sess)
                .await?;
        }
        Err(AccessError::NotLogin) => {}
        Err(err) => return Err(err.into()),
    }
    Ok(JsonResponse::default())
}
