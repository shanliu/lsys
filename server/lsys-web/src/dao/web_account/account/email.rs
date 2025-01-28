use super::WebUserAccount;
use crate::common::{CaptchaParam, JsonData, JsonError, JsonResult};
use lsys_access::dao::SessionBody;
use lsys_core::{fluent_message, RequestEnv};
use lsys_user::model::AccountEmailStatus;

impl WebUserAccount {
    //添加用户邮箱
    pub async fn user_email_add(
        &self,
        email: &str,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;
        let status = lsys_user::model::AccountEmailStatus::Init;
        let email_id = self
            .user_dao
            .account_dao
            .account_email
            .add_email(
                &account,
                email,
                status,
                session_body.user_id(),
                None,
                env_data,
            )
            .await?;
        Ok(email_id)
    }
}

impl WebUserAccount {
    //发送邮箱验证码
    pub async fn user_email_send_code(
        &self,
        email: &str,
        captcha: &CaptchaParam,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        let valid_code = self
            .captcha
            .valid_code(&crate::dao::CaptchaKey::AddEmailCode);
        valid_code.check_code(&captcha.into()).await?;
        let email_res = self
            .user_dao
            .account_dao
            .account_email
            .find_by_last_email(email)
            .await;
        let email = match email_res {
            Ok(email) => {
                if AccountEmailStatus::Valid.eq(email.status) {
                    if email.account_id != session_body.user_id() {
                        return Err(JsonError::JsonData(
                            JsonData::default().set_code(500),
                            fluent_message!("mail-bind-other-user",{
                                "other.account_id":email.account_id,
                                "account_id":session_body.user_id()
                            }),
                        ));
                    } else {
                        return Err(JsonError::JsonData(
                            JsonData::default()
                                .set_code(500)
                                .set_sub_code("mail-is-confirm"),
                            fluent_message!("mail-is-confirm"),
                        ));
                    }
                }
                email
            }
            Err(err) => {
                if !err.is_not_found() {
                    return Err(err.into());
                } else {
                    return Ok(());
                }
            }
        };

        let res = self
            .user_dao
            .account_dao
            .account_email
            .valid_code_set(
                &mut self.user_dao.account_dao.account_email.valid_code_builder(),
                &email.account_id,
                &email.email,
            )
            .await?;
        self.sender
            .mailer
            .send_valid_code(&email.email, &res.0, &res.1, env_data)
            .await?;
        Ok(())
    }
}
