use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResult, UserAuthQueryDao},
    dao::{
        access::api::system::auth::CheckSystemLogin, OauthCallbackParam, OauthLogin,
        OauthLoginParam, ShowUserAuthData,
    },
};
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
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::any(&req_dao.req_env),
            &CheckSystemLogin {},
        )
        .await?;

    req_dao
        .web_dao
        .web_user
        .auth
        .external_login(
            oauth,
            param,
            op_user_id,
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await
}
