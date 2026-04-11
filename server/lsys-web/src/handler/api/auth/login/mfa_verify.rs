use std::net::IpAddr;

use crate::{
    common::{JsonResult, UserAuthQueryDao},
    dao::ShowUserAuthData,
};

use lsys_user::dao::{UserAuthToken, login::AccountLoginEnv};
use serde::Deserialize;

use super::local_login::user_login_finish;

#[derive(Debug, Deserialize)]
pub struct MfaVerifyParam {
    pub mfa_token: String,
    pub code: String,
}

pub async fn user_mfa_verify(
    param: &MfaVerifyParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
    let session_body = req_dao
        .web_dao
        .web_user
        .user_dao
        .mfa_login_dao
        .verify_totp_and_login(
            &param.mfa_token,
            &param.code,
            &AccountLoginEnv {
                login_ip: req_dao
                    .req_env
                    .request_ip
                    .as_ref()
                    .and_then(|e| e.parse::<IpAddr>().ok()),
            },
        )
        .await?;

    user_login_finish(session_body, req_dao).await
}
