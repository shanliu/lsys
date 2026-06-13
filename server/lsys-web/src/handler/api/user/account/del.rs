use crate::common::JsonResponse;
use crate::common::{JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use lsys_access::dao::AccessSession;
use lsys_access::dao::AccessSessionData;
use lsys_core::fluent_message;
use lsys_user::dao::{AccountError, UserAuthToken};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct DeleteParam {
    pub password: String,
}
//删除用户
pub async fn delete(
    param: &DeleteParam,
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
    let account = web_dao
        .web_user
        .user_dao
        .account_dao
        .session_account(auth_data.session_body())
        .await?;
    if web_dao
        .web_user
        .user_dao
        .account_dao
        .account_password
        .check_password(&account, &param.password)
        .await?
    {
        return Err(AccountError::PasswordNotMatch((
            auth_data.user_id(),
            fluent_message!("auth-bad-password"), //" bad password"
        ))
        .into());
    }
    web_dao
        .web_user
        .account
        .user_delete_from_session(&auth_data, Some(&req_dao.req_env))
        .await?;
    auth_dao
        .user_session
        .write()
        .await
        .set_session_token(UserAuthToken::default());
    Ok(JsonResponse::default())
}
