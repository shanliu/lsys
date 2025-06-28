use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonData, JsonResponse, JsonResult, RequestDao},
    dao::{
        access::api::system::auth::CheckSystemLogin, OauthCallbackParam, OauthLogin,
        OauthLoginParam,
    },
};
use serde::Serialize;
use serde_json::json;
//检查权限并获取登录URL
pub async fn user_external_login_url<
    T: OauthLogin<L, P, D>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>(
    oauth: &T,
    param: &L,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::any(&req_dao.req_env),
            &CheckSystemLogin {},
        )
        .await?;
    let url = req_dao
        .web_dao
        .web_user
        .auth
        .oauth_user_login_url::<T, L, P, D>(oauth, param)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "url": url }))))
}
