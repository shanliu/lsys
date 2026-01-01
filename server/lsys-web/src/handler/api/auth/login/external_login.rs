use crate::{
    common::{JsonResult, UserAuthQueryDao},
    dao::{OauthCallbackParam, OauthLogin, OauthLoginParam, ShowUserAuthData},
};
use lsys_access::dao::AccessSession;
use lsys_user::dao::UserAuthToken;
use serde::Serialize;
//检查权限并完成回调
pub async fn user_login_from_external<
    O: OauthLogin<L, P, Q>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    Q: Serialize + Send + Sync,
>(
    oauth: &O,
    param: &P,
    op_user_id: u64,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let session_body = req_dao
        .web_dao
        .web_user
        .auth
        .external_login(oauth, param, op_user_id, Some(&req_dao.req_env))
        .await?;
    let user_token = UserAuthToken::new(
        session_body.session().user_app_id,
        session_body.token_data(),
        session_body.user_id(),
        session_body.session().expire_time,
    );
    req_dao
        .user_session
        .write()
        .await
        .set_session_token(user_token.clone());
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    Ok((
        user_token,
        req_dao
            .web_dao
            .web_user
            .auth
            .create_show_account_auth_data(&auth_data)
            .await?,
    ))
}
