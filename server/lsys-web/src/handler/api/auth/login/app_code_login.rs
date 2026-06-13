use crate::{
    common::{CaptchaParam, JsonResult, RequestDao, UserAuthQueryDao},
    dao::{
        ShowUserAuthData, WebDao,
        access::{RbacAccessCheckEnv, api::system::auth::CheckSystemLogin},
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
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let app = web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_client_id(&param.client_id)
        .await?;
    //需要检查应用是否支持code登陆
    //只有系统应用才能code登陆
    web_dao
        .web_app
        .app_dao
        .exter_login
        .inner_feature_exter_login_check(&app)
        .await?;

    let session_body = web_dao
        .web_user
        .auth
        .app_code_login(
            app.id,
            &param.token_data,
            param.captcha.as_ref(),
            Some(&req_dao.req_env),
        )
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&session_body, &req_dao.req_env),
            &CheckSystemLogin {},
        )
        .await?;

    let user_token = AuthCode::to_token(&session_body);
    auth_dao
        .user_session
        .write()
        .await
        .set_session_token(user_token.to_owned());
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    Ok((
        user_token,
        web_dao
            .web_user
            .auth
            .create_show_account_auth_data(&auth_data)
            .await?,
    ))
}
