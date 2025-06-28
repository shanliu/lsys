use crate::{
    common::{CaptchaParam, JsonResult, UserAuthQueryDao},
    dao::{
        access::{api::system::auth::CheckSystemLogin, RbacAccessCheckEnv},
        ShowUserAuthData,
    },
};

use lsys_user::dao::UserAuthToken;
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
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::any(&req_dao.req_env),
            &CheckSystemLogin {},
        )
        .await?;

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

    req_dao
        .web_dao
        .web_user
        .auth
        .app_code_login(
            app.id,
            &param.token_data,
            param.captcha.as_ref(),
            &req_dao.user_session,
            Some(&req_dao.req_env),
        )
        .await
}
