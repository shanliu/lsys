//用户注册的封装
use lsys_core::RequestEnv;
use lsys_user::{
    dao::AccountResult,
    model::{AccountEmailStatus, AccountInfoModelRef, AccountMobileStatus, AccountModel},
};

use lsys_core::fluent_message;
use lsys_core::model_option_set;

use crate::common::{CaptchaParam, JsonData, JsonError, JsonResult};

use super::WebUserAuth;

pub struct AccountRegData<'t> {
    pub status_enable: bool,
    pub nikename: &'t str,
    pub passwrod: Option<&'t str>,
    pub name: Option<&'t str>,
    pub email: Option<(&'t str, AccountEmailStatus)>,
    pub mobile: Option<(&'t str, &'t str, AccountMobileStatus)>,
    pub external: Option<(&'t str, &'t str, &'t str, &'t str)>,
    pub info: Option<AccountInfoModelRef<'t>>,
}

impl WebUserAuth {
    // 注册用户
    pub async fn reg_user(
        &self,
        reg_data: &AccountRegData<'_>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<AccountModel> {
        let mut tran = self.db.begin().await?;
        let user = match self
            .user_dao
            .account_dao
            .account
            .add(reg_data.nikename, op_user_id, Some(&mut tran), env_data)
            .await
        {
            Ok(u) => u,
            Err(err) => {
                tran.rollback().await?;
                return Err(err);
            }
        };
        if reg_data.status_enable {
            let res = self
                .user_dao
                .account_dao
                .account
                .enable(&user, op_user_id, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some(pw) = reg_data.passwrod {
            let res = self
                .user_dao
                .account_dao
                .account_password
                .set_passwrod(&user, pw, op_user_id, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some((un, st)) = reg_data.email {
            let res = self
                .user_dao
                .account_dao
                .account_email
                .add_email(&user, un, st, op_user_id, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some(un) = reg_data.name {
            let res = self
                .user_dao
                .account_dao
                .account_name
                .change_account_name(&user, un, op_user_id, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some((area, mob, st)) = reg_data.mobile {
            let res = self
                .user_dao
                .account_dao
                .account_mobile
                .add_mobile(&user, area, mob, st, op_user_id, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some((config_name, external_type, external_id, external_name)) = reg_data.external {
            let res = self
                .user_dao
                .account_dao
                .account_external
                .add_external(
                    &user,
                    config_name,
                    external_type,
                    external_id,
                    external_name,
                    op_user_id,
                    Some(&mut tran),
                    env_data,
                )
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some(ref info_ref) = reg_data.info {
            let res = self
                .user_dao
                .account_dao
                .account_info
                .set_info(&user, info_ref, op_user_id, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        tran.commit().await?;
        Ok(user)
    }
}

pub struct RegFromNameData<'t> {
    pub nikename: Option<&'t str>,
    pub name: &'t str,
    pub password: &'t str,
}
impl WebUserAuth {
    // 通过用户名注册
    pub async fn user_reg_from_name(
        &self,
        param: &RegFromNameData<'_>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let reg_ip = env_data
            .map(|e| e.request_ip.clone().unwrap_or_default())
            .unwrap_or_default();
        let info = model_option_set!(AccountInfoModelRef,{
            reg_ip:  reg_ip,
        });
        let user = self
            .reg_user(
                &AccountRegData {
                    status_enable: true,
                    nikename: param.nikename.unwrap_or(param.name),
                    passwrod: Some(param.password),
                    name: Some(param.name),
                    email: None,
                    mobile: None,
                    external: None,
                    info: Some(info),
                },
                op_user_id,
                env_data,
            )
            .await?;
        Ok(user.id)
    }
}
pub struct RegSendCodeFromMobileData<'t> {
    pub mobile: &'t str,
    pub area_code: &'t str,
    pub captcha: &'t CaptchaParam,
}
impl WebUserAuth {
    // 发送注册用户短信验证码
    pub async fn user_reg_send_code_from_mobile(
        &self,
        param: &RegSendCodeFromMobileData<'_>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<usize> {
        let valid_code = self.captcha.valid_code(&crate::dao::CaptchaKey::RegSms);
        valid_code.check_code(&param.captcha.into()).await?;
        let mobile_res = self
            .user_dao
            .account_dao
            .account_mobile
            .find_by_last_mobile(param.area_code, param.mobile)
            .await;
        if let Ok(mobile) = mobile_res {
            if AccountMobileStatus::Valid.eq(mobile.status) {
                return Err(JsonError::JsonResponse(
                    JsonData::default().set_sub_code("mobile_is_reg"),
                    fluent_message!("reg-mobile-registered"),
                ));
            }
        }
        let data = self
            .user_dao
            .account_dao
            .account_mobile
            .valid_code_set(
                &mut self
                    .user_dao
                    .account_dao
                    .account_mobile
                    .valid_code_builder(),
                param.area_code,
                param.mobile,
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
pub struct RegSendCodeFromEmailData<'t> {
    pub email: &'t str,
    pub captcha: &'t CaptchaParam,
}
impl WebUserAuth {
    // 发送注册用户邮箱验证码
    pub async fn user_reg_send_code_from_email(
        &self,
        param: &RegSendCodeFromEmailData<'_>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<usize> {
        let valid_code = self.captcha.valid_code(&crate::dao::CaptchaKey::RegEmail);
        let email_res = self
            .user_dao
            .account_dao
            .account_email
            .find_by_last_email(param.email)
            .await;
        if let Ok(email) = email_res {
            if AccountEmailStatus::Valid.eq(email.status) {
                return Err(JsonError::JsonResponse(
                    JsonData::default().set_sub_code("mobile_is_reg"),
                    fluent_message!("reg-mobile-registered"),
                ));
            }
        }
        valid_code.check_code(&param.captcha.into()).await?;
        let data = self
            .user_dao
            .account_dao
            .account_email
            .valid_code_set(
                &mut self.user_dao.account_dao.account_email.valid_code_builder(),
                0,
                param.email,
            )
            .await?;
        self.sender
            .mailer
            .send_valid_code(param.email, &data.0, &data.1, env_data)
            .await?;
        self.captcha.destroy_code(&valid_code, param.email).await;

        Ok(data.1)
    }
}
pub struct RegFromEmailData<'t> {
    pub email: &'t str,
    pub code: &'t str,
    pub password: &'t str,
    pub nikename: &'t str,
}
impl WebUserAuth {
    // 通过邮件验证码注册
    pub async fn user_reg_from_email(
        &self,
        param: &RegFromEmailData<'_>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        self.user_dao
            .account_dao
            .account_email
            .valid_code_check(param.code, 0, param.email)
            .await?;
        let reg_ip = env_data
            .map(|e| e.request_ip.clone().unwrap_or_default())
            .unwrap_or_default();
        let info = model_option_set!(AccountInfoModelRef,{
            reg_ip:reg_ip,
        });
        let user = self
            .reg_user(
                &AccountRegData {
                    status_enable: true,
                    nikename: param.nikename,
                    passwrod: Some(param.password),
                    name: None,
                    email: Some((param.email, AccountEmailStatus::Valid)),
                    mobile: None,
                    external: None,
                    info: Some(info),
                },
                op_user_id,
                env_data,
            )
            .await?;
        let _ = self
            .user_dao
            .account_dao
            .account_email
            .valid_code_clear(0, param.email)
            .await;
        Ok(user.id)
    }
}
pub struct RegFromMobileData<'t> {
    pub mobile: &'t str,
    pub area_code: &'t str,
    pub code: &'t str,
    pub password: &'t str,
    pub nikename: &'t str,
}
impl WebUserAuth {
    // 通过短信验证码注册
    pub async fn user_reg_from_mobile(
        &self,
        param: &RegFromMobileData<'_>,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        self.user_dao
            .account_dao
            .account_mobile
            .valid_code_check(param.code, param.area_code, param.mobile)
            .await?;
        let reg_ip = env_data
            .map(|e| e.request_ip.clone().unwrap_or_default())
            .unwrap_or_default();
        let info = model_option_set!(AccountInfoModelRef,{
            reg_ip:reg_ip,
        });
        let user = self
            .reg_user(
                &AccountRegData {
                    status_enable: true,
                    nikename: param.nikename,
                    passwrod: Some(param.password),
                    name: None,
                    email: None,
                    mobile: Some((param.area_code, param.mobile, AccountMobileStatus::Valid)),
                    external: None,
                    info: Some(info),
                },
                op_user_id,
                env_data,
            )
            .await?;
        let _ = self
            .user_dao
            .account_dao
            .account_mobile
            .valid_code_clear(param.area_code, param.mobile)
            .await;
        Ok(user.id)
    }
}
