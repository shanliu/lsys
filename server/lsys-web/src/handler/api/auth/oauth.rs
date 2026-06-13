use crate::{
    common::{JsonData, JsonResponse, JsonResult},
    dao::{OauthCallbackParam, OauthLogin, OauthLoginParam, WebDao},
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

    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let url = web_dao
        .web_user
        .auth
        .oauth_user_login_url::<T, L, P, D>(oauth, param)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "url": url }))))
}
