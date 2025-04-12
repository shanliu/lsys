// 密码修改相关操作封装
use lsys_access::dao::SessionBody;
use lsys_core::{fluent_message, RequestEnv};

use super::WebUserAuth;
use crate::common::JsonData;
use crate::common::{CaptchaParam, JsonError, JsonResult};

impl WebUserAuth {
    //统一重置密码
    async fn reset_password(
        &self,
        account_id: u64,
        new_password: &str,
        from_type: &str,
        code: &str,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let user = self
            .user_dao
            .account_dao
            .account
            .find_by_id(&account_id)
            .await?;
        user.is_enable()?;

        let pid = self
            .user_dao
            .account_dao
            .account_password
            .set_passwrod_from_code(
                &user,
                new_password,
                from_type,
                code,
                op_user_id,
                None,
                env_data,
            )
            .await?;
        Ok(pid)
    }
}

pub struct SetPasswordData<'t> {
    pub old_password: Option<&'t str>,
    pub new_password: &'t str,
}
impl WebUserAuth {
    //用户设置密码
    pub async fn user_set_password(
        &self,
        param: &SetPasswordData<'_>,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;

        if account.password_id > 0 {
            if let Some(old_passwrod) = param.old_password {
                let check = self
                    .user_dao
                    .account_dao
                    .account_password
                    .check_password(&account, old_passwrod)
                    .await?;
                if !check {
                    return Err(JsonError::JsonResponse(
                        JsonData::default().set_sub_code("bad_passwrod"),
                        fluent_message!("user-old-passwrod-bad"),
                    ));
                }
            } else {
                return Err(JsonError::JsonResponse(
                    JsonData::default().set_sub_code("need_old_passwrod"),
                    fluent_message!("user-old-passwrod-empty"),
                ));
            }
        }
        let pid = self
            .user_dao
            .account_dao
            .account_password
            .set_passwrod(
                &account,
                param.new_password,
                session_body.user_id(),
                None,
                env_data,
            )
            .await?;
        Ok(pid)
    }
}

pub struct ResetPasswordSendCodeFromMobileData<'t> {
    pub mobile: &'t str,
    pub area_code: &'t str,
    pub captcha: &'t CaptchaParam,
}
impl WebUserAuth {
    //通过短信发送重置密码验证码
    pub async fn user_reset_password_send_code_from_mobile(
        &self,
        param: &ResetPasswordSendCodeFromMobileData<'_>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<usize> {
        let mobile = self
            .user_dao
            .account_dao
            .account_mobile
            .find_by_last_mobile(param.area_code, param.mobile)
            .await?;
        mobile.is_enable()?;
        let valid_code = self
            .captcha
            .valid_code(&crate::dao::CaptchaKey::ResetPasswordSms);

        valid_code.check_code(&param.captcha.into()).await?;
        let data = self
            .user_dao
            .account_dao
            .account_password
            .valid_code_set(
                &mut self
                    .user_dao
                    .account_dao
                    .account_password
                    .valid_code_builder(),
                &mobile.account_id,
                &format!("mobile-{}-{}", param.area_code, param.mobile),
            )
            .await?;
        self.sender
            .smser
            .send_valid_code(param.area_code, param.mobile, &data.0, &data.1, env_data)
            .await?;
        self.captcha
            .destroy_code(&valid_code, &param.captcha.key)
            .await;
        Ok(data.1)
    }
}

pub struct ResetPasswordSendCodeFromEmailData<'t> {
    pub email: &'t str,
    pub captcha: &'t CaptchaParam,
}
impl WebUserAuth {
    //通过邮箱发送重置密码验证码
    pub async fn user_reset_password_send_code_from_email(
        &self,
        param: &ResetPasswordSendCodeFromEmailData<'_>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<usize> {
        let user_email = self
            .user_dao
            .account_dao
            .account_email
            .find_by_last_email(param.email)
            .await?;
        user_email.is_enable()?;
        let valid_code = self
            .captcha
            .valid_code(&crate::dao::CaptchaKey::ResetPasswordMail);
        valid_code.check_code(&param.captcha.into()).await?;
        let data = self
            .user_dao
            .account_dao
            .account_password
            .valid_code_set(
                &mut self
                    .user_dao
                    .account_dao
                    .account_password
                    .valid_code_builder(),
                &user_email.account_id,
                &format!("mail-{}", param.email),
            )
            .await?;
        self.sender
            .mailer
            .send_valid_code(param.email, &data.0, &data.1, env_data)
            .await?;
        self.captcha
            .destroy_code(&valid_code, &param.captcha.key)
            .await;
        Ok(data.1)
    }
}

pub struct ResetPasswordFromEmailData<'t> {
    pub email: &'t str,
    pub code: &'t str,
    pub new_password: &'t str,
}
impl WebUserAuth {
    //通过邮箱验证码重置密码
    pub async fn user_reset_password_from_email(
        &self,
        param: &ResetPasswordFromEmailData<'_>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let email = self
            .user_dao
            .account_dao
            .account_email
            .find_by_last_email(param.email)
            .await?;
        email.is_enable()?;
        self.reset_password(
            email.account_id,
            param.new_password,
            &format!("mail-{}", param.email),
            param.code,
            op_user_id,
            env_data,
        )
        .await
    }
}

pub struct ResetPasswordFromMobileData<'t> {
    pub area_code: &'t str,
    pub mobile: &'t str,
    pub code: &'t str,
    pub new_password: &'t str,
}
impl WebUserAuth {
    //通过短信验证码重置密码
    pub async fn user_reset_password_from_mobile(
        &self,
        param: &ResetPasswordFromMobileData<'_>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let mobile = self
            .user_dao
            .account_dao
            .account_mobile
            .find_by_last_mobile(param.area_code, param.mobile)
            .await?;
        mobile.is_enable()?;
        self.reset_password(
            mobile.account_id,
            param.new_password,
            &format!("mobile-{}-{}", param.area_code, param.mobile),
            param.code,
            op_user_id,
            env_data,
        )
        .await
    }
}
