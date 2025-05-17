use super::super::AccountLoginEnv;
use super::{reload_match_wrap, AccountLoginMeta, AccountLoginParam};

use crate::dao::account::AccountError;

use crate::dao::{AccountDao, AccountResult, UserAuthData, UserAuthResult, UserLoginReload};
use crate::model::{AccountEmailModel, AccountModel};
use async_trait::async_trait;
use lsys_access::dao::SessionBody;
use lsys_core::{
    fluent_message, valid_key, IntoFluentMessage, ValidEmail, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use serde_json::{json, Value};
use std::sync::Arc;

use tracing::warn;

pub struct EmailCodeLoginData {
    data: AccountEmailModel,
}

impl EmailCodeLoginData {
    pub fn new(data: AccountEmailModel) -> Self {
        EmailCodeLoginData { data }
    }
    pub fn to_json(&self) -> Value {
        json!(self.data)
    }
    pub async fn from(account_dao: &AccountDao, auth_data: &UserAuthData) -> AccountResult<Self> {
        match auth_data.login_data().parse::<u64>() {
            Err(err) => Err(AccountError::System(fluent_message!(
                "account-bad-session",
                err
            ))),
            Ok(id) => {
                let data = account_dao.account_email.cache().find_by_id(&id).await?;
                data.is_enable()?;
                Ok(EmailCodeLoginData::new(data))
            }
        }
    }
}

impl std::ops::Deref for EmailCodeLoginData {
    type Target = AccountEmailModel;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct EmailCodeLoginReload {
    account_dao: Arc<AccountDao>,
}

impl EmailCodeLoginReload {
    pub fn new(account_dao: Arc<AccountDao>) -> Self {
        Self { account_dao }
    }
}

#[async_trait]
impl UserLoginReload for EmailCodeLoginReload {
    async fn reload(
        &self,
        session: &SessionBody,
        data: &str,
    ) -> Option<UserAuthResult<(String, u64)>> {
        reload_match_wrap::<EmailCodeLoginMeta>(session, async {
            let id = data
                .parse::<u64>()
                .map_err(|err| AccountError::System(fluent_message!("auth-bad-session", err)))?;
            let dat = self.account_dao.account_email.find_by_id(&id).await?;
            dat.is_enable()?;
            Ok(dat.id.to_string())
        })
        .await
    }
}

pub struct EmailCodeLoginMeta {}
impl AccountLoginMeta for EmailCodeLoginMeta {
    fn login_type() -> String {
        "email-code".to_string()
    }
}

pub struct EmailCodeLogin {
    redis: deadpool_redis::Pool,
    account: Arc<AccountDao>,
    pub email: String,
    pub code: String,
}

impl EmailCodeLogin {
    /// 验证码生成
    fn valid_code(redis: deadpool_redis::Pool) -> lsys_core::ValidCode {
        lsys_core::ValidCode::new(redis, "email-login", true)
    }
    async fn email_param_valid(email: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("login_email"),
                &email,
                &ValidParamCheck::default()
                    .add_rule(ValidEmail::default())
                    .add_rule(ValidStrlen::range(3, 150)),
            )
            .check()?;
        Ok(())
    }
    /// 获取验证码
    pub async fn valid_code_set<T: lsys_core::ValidCodeData>(
        redis: deadpool_redis::Pool,
        valid_code_data: &mut T,
        email: &str,
    ) -> AccountResult<(String, usize)> {
        Self::email_param_valid(email).await?;
        let valid_code = Self::valid_code(redis);
        let code = valid_code.set_code(email, valid_code_data).await?;
        Ok(code)
    }
    /// 验证码构造器
    pub fn valid_code_builder() -> lsys_core::ValidCodeDataRandom {
        lsys_core::ValidCodeDataRandom::new(300, 30)
    }
    /// 检测验证码
    pub async fn valid_code_check(
        redis: deadpool_redis::Pool,
        code: &str,
        email: &str,
    ) -> AccountResult<()> {
        Self::valid_code(redis)
            .check_code(&lsys_core::CheckCodeData::new(email, code))
            .await?;
        Ok(())
    }
    pub async fn valid_code_clear(redis: deadpool_redis::Pool, email: &str) -> AccountResult<()> {
        let mut builder = Self::valid_code_builder();
        Self::valid_code(redis)
            .destroy_code(email, &mut builder)
            .await?;
        Ok(())
    }
}
impl EmailCodeLogin {
    async fn new_param_valid(email: &str, code: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("login_email"),
                &email,
                &ValidParamCheck::default().add_rule(ValidEmail::default()),
            )
            .add(
                valid_key!("login_code"),
                &code,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Alphanumeric)
                    .add_rule(ValidStrlen::range(2, 8)),
            )
            .check()?;
        Ok(())
    }
    pub async fn new(
        redis: deadpool_redis::Pool,
        account: Arc<AccountDao>,
        email: &str,
        code: &str,
    ) -> AccountResult<Self> {
        Self::new_param_valid(email, code).await?;
        Ok(Self {
            redis,
            account,
            email: email.to_string(),
            code: code.to_string(),
        })
    }
}
#[async_trait]
impl AccountLoginParam for EmailCodeLogin {
    type Meta = EmailCodeLoginMeta;
    fn account_name(&self) -> String {
        self.email.clone()
    }
    async fn get_account(&self, _: &AccountLoginEnv) -> AccountResult<(String, AccountModel)> {
        let email = self
            .account
            .account_email
            .find_by_last_email(&self.email)
            .await
            .map_err(auth_user_not_found_map!(self.account_name(), "email code"))?;
        email.is_enable()?;

        Self::valid_code_check(self.redis.to_owned(), &self.code, &self.email).await?;

        let user = self
            .account
            .account
            .find_by_id(&email.account_id)
            .await
            .map_err(auth_user_not_found_map!(
                self.account_name(),
                "email code [user id]"
            ))?;
        user.is_enable()?;

        if let Err(err) = Self::valid_code_clear(self.redis.to_owned(), &self.email).await {
            warn!(
                "login email clear valid[{}] fail:{}",
                self.email,
                err.to_fluent_message().default_format()
            )
        }

        Ok((email.id.to_string(), user))
    }
}
