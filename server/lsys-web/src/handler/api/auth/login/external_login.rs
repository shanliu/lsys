use crate::{
    common::{JsonResult, RequestDao, UserAuthQueryDao},
    dao::{OauthCallbackParam, OauthLogin, OauthLoginParam, WebDao},
};
use serde::Serialize;

use super::local_login::user_login_finish;
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
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(lsys_user::dao::UserAuthToken, crate::dao::ShowUserAuthData)> {
    let session_body = web_dao
        .web_user
        .auth
        .external_login(oauth, param, op_user_id, Some(&req_dao.req_env))
        .await?;

    user_login_finish(session_body, req_dao, auth_dao, web_dao).await
}
