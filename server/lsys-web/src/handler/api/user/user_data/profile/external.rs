use crate::{
    common::{
        JsonData, JsonResult, OauthCallbackParam, OauthLogin, OauthLoginParam, RequestDao,
        UserAuthQueryDao,
    },
    dao::access::{api::user::CheckUserExternalEdit, common::CheckSystemLogin},
};

use lsys_access::dao::AccessSession;

use serde::{Deserialize, Serialize};
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ExternalDeleteParam {
    pub ext_id: u64,
}
pub async fn user_external_delete(
    param: &ExternalDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.access_env().await?,
            &CheckUserExternalEdit {
                res_user_id: req_dao
                    .web_dao
                    .web_user
                    .account
                    .account_id_to_user(ext.account_id)
                    .await?
                    .id,
            },
            None,
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
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct ExternalListDataParam {
    pub oauth_type: Option<Vec<String>>,
}

pub async fn user_external_list_data(
    param: &ExternalListDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
    Ok(JsonData::data(json!({
        "data": data ,
        "total":data.len(),
    })))
}

//检查权限并获取登录URL
pub async fn user_external_bind_url<
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
        .check(&req_dao.access_env(), &CheckSystemLogin {}, None)
        .await?;
    let url = req_dao
        .web_dao
        .web_user
        .auth
        .oauth_user_login_url::<T, L, P, D>(oauth, param)
        .await?;
    Ok(JsonData::data(json!({ "url": url })))
}

/// 已登陆后绑定外部账号
pub async fn user_external_bind<
    T: OauthLogin<L, P, D>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>(
    oauth: &T,
    param: &P,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckSystemLogin {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
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
    Ok(JsonData::data(json!({ "id": ext_model.id })))
}
