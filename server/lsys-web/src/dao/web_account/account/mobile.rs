use crate::common::{CaptchaParam, JsonData, JsonError, JsonResult};
use lsys_access::dao::SessionBody;
use lsys_core::{fluent_message, RequestEnv};
use lsys_user::model::AccountMobileStatus;

use super::WebUserAccount;

impl WebUserAccount {
    pub async fn user_mobile_add(
        &self,
        area_code: &str,
        mobile: &str,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;
        let mobile_id = self
            .user_dao
            .account_dao
            .account_mobile
            .add_mobile(
                &account,
                area_code,
                mobile,
                AccountMobileStatus::Init,
                session_body.user_id(),
                None,
                env_data,
            )
            .await?;
        Ok(mobile_id)
    }
}

impl WebUserAccount {
    pub async fn user_mobile_send_code(
        &self,
        area_code: &str,
        mobile: &str,
        captcha: &CaptchaParam,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<usize> {
        let valid_code = self.captcha.valid_code(&crate::dao::CaptchaKey::AddSmsCode);
        valid_code.check_code(&captcha.into()).await?;

        let mobile_res = self
            .user_dao
            .account_dao
            .account_mobile
            .find_by_last_mobile(area_code, mobile)
            .await;
        match mobile_res {
            Ok(mobile) => {
                if AccountMobileStatus::Valid.eq(mobile.status) {
                    if mobile.account_id != session_body.user_id() {
                        return Err(JsonError::JsonResponse(
                            JsonData::default(),
                            fluent_message!("mobile-bind-other-user",{
                                "id": mobile.account_id
                            }),
                        ));
                    } else {
                        return Err(JsonError::JsonResponse(
                            JsonData::default(),
                            fluent_message!("mobile-is-bind"),
                        ));
                    }
                }
            }
            Err(err) => {
                if !err.is_not_found() {
                    return Err(err.into());
                }
            }
        };

        let (code, ttl) = self
            .user_dao
            .account_dao
            .account_mobile
            .valid_code_set(
                &mut self
                    .user_dao
                    .account_dao
                    .account_mobile
                    .valid_code_builder(),
                area_code,
                mobile,
            )
            .await?;
        self.sender
            .smser
            .send_valid_code(area_code, mobile, &code, &ttl, env_data)
            .await?;
        Ok(ttl)
    }
}
