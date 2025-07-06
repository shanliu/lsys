use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao},
    dao::{
        access::api::system::{auth::CheckSystemLogin, user::CheckUserExternalEdit},
        OauthCallbackParam, OauthLogin, OauthLoginParam,
    },
};
use lsys_access::dao::AccessSession;

use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ExternalDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub ext_id: u64,
}
pub async fn external_delete(
    param: &ExternalDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let ext = req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_external
        .find_by_id(&param.ext_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserExternalEdit {
                res_user_id: req_dao
                    .web_dao
                    .web_user
                    .account
                    .account_id_to_user(ext.account_id)
                    .await?
                    .id,
            },
        )
        .await?;
    req_dao
        .web_dao
        .web_user
        .user_dao
        .account_dao
        .account_external
        .del_external(&ext, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ExternalListDataParam {
    pub oauth_type: Option<Vec<String>>,
}

pub async fn external_list_data(
    param: &ExternalListDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let otype = param
        .oauth_type
        .as_ref()
        .map(|e| e.iter().map(|e| e.as_str()).collect::<Vec<_>>());
    let data = req_dao
        .web_dao
        .web_user
        .account
        .user_external(auth_data.user_id(), otype.as_ref().map(|e| e.as_ref()))
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": data ,
        "total":data.len(),
    }))))
}

//检查权限并获取登录URL
pub async fn external_bind_url<
    T: OauthLogin<L, P, D>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>(
    oauth: &T,
    param: &L,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    let url = req_dao
        .web_dao
        .web_user
        .auth
        .oauth_user_login_url::<T, L, P, D>(oauth, param)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "url": url }))))
}

/// 已登陆后绑定外部账号
pub async fn external_bind<
    T: OauthLogin<L, P, D>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>(
    oauth: &T,
    param: &P,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckSystemLogin {},
        )
        .await?;
    let (ext_model, _, _) = &req_dao
        .web_dao
        .web_user
        .auth
        .oauth_user_bind(
            oauth,
            param,
            auth_data.account_id()?,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(
        json!({ "id": ext_model.id }),
    )))
}
