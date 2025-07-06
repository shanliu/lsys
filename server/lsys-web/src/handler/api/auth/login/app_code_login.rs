use crate::{
    common::{CaptchaParam, JsonResult, UserAuthQueryDao},
    dao::{
        access::{api::system::auth::CheckSystemLogin, RbacAccessCheckEnv},
        ShowUserAuthData,
    },
};
use lsys_access::dao::AccessSession;
use lsys_user::dao::{AuthCode, UserAuthToken};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppCodeLoginParam {
    client_id: String,
    token_data: String,
    captcha: Option<CaptchaParam>,
}
pub async fn user_login_from_app_code(
    param: &AppCodeLoginParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&param.client_id)
        .await?;
    //需要检查应用是否支持code登陆
    //只有系统应用才能code登陆
    req_dao
        .web_dao
        .web_app
        .app_dao
        .exter_login
        .inner_feature_exter_login_check(&app)
        .await?;

    let session_body = req_dao
        .web_dao
        .web_user
        .auth
        .app_code_login(
            app.id,
            &param.token_data,
            param.captcha.as_ref(),
            Some(&req_dao.req_env),
        )
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&session_body, &req_dao.req_env),
            &CheckSystemLogin {},
        )
        .await?;

    let user_token = AuthCode::to_token(&session_body);
    req_dao
        .user_session
        .write()
        .await
        .set_session_token(user_token.to_owned());
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
