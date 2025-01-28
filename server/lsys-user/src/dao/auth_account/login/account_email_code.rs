use super::super::AccountLoginEnv;
use super::{reload_match_wrap, AccountLoginMeta, AccountLoginParam};

use crate::dao::account::AccountError;

use crate::dao::{AccountDao, AccountResult, UserAuthData, UserAuthResult, UserLoginReload};
use crate::model::{AccountEmailModel, AccountModel};
use async_trait::async_trait;
use lsys_access::dao::SessionBody;
use lsys_core::{fluent_message, IntoFluentMessage};
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
    impl_auth_valid_code_method!("email-login",{
        email: &str,
    },{
        email.to_owned()
    },5*60);
}
impl EmailCodeLogin {
    pub fn new(
        redis: deadpool_redis::Pool,
        account: Arc<AccountDao>,
        email: &str,
        code: &str,
    ) -> Self {
        Self {
            redis,
            account,
            email: email.to_string(),
            code: code.to_string(),
        }
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
