use crate::{
    common::{JsonData, JsonResult, OauthCallbackParam, OauthLogin, OauthLoginParam, RequestDao},
    dao::access::api::auth::CheckSystemLogin,
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
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckSystemLogin {})
        .await?;
    let url = req_dao
        .web_dao
        .web_user
        .auth
        .oauth_user_login_url::<T, L, P, D>(oauth, param)
        .await?;
    Ok(JsonData::data(json!({ "url": url })))
}
