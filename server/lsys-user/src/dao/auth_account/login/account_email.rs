use super::super::AccountLoginEnv;
use super::{auth_check_account_password, reload_match_wrap, AccountLoginMeta, AccountLoginParam};
use crate::dao::auth::UserLoginReload;
use crate::dao::{AccountDao, AccountError, AccountResult, UserAuthData, UserAuthResult};

use crate::model::{AccountEmailModel, AccountModel};
use async_trait::async_trait;
use lsys_access::dao::SessionBody;
use lsys_core::{fluent_message, valid_key, ValidEmail, ValidParam, ValidParamCheck, ValidStrlen};
use serde_json::{json, Value};

use std::sync::Arc;

pub struct EmailLoginData {
    data: AccountEmailModel,
}

impl EmailLoginData {
    pub fn new(data: AccountEmailModel) -> Self {
        EmailLoginData { data }
    }
    pub fn to_json(&self) -> Value {
        json!(self.data)
    }
    pub async fn from(account_dao: &AccountDao, auth_data: &UserAuthData) -> AccountResult<Self> {
        match auth_data.login_data().parse::<u64>() {
            Err(err) => Err(AccountError::System(fluent_message!(
                "account-bad-session",{
                    "msg":err,
                    "data":auth_data.login_data()
                }
            ))),
            Ok(id) => {
                let data = account_dao.account_email.cache().find_by_id(&id).await?;
                data.is_enable()?;
                Ok(EmailLoginData::new(data))
            }
        }
    }
}

impl std::ops::Deref for EmailLoginData {
    type Target = AccountEmailModel;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct EmailLoginReload {
    account_dao: Arc<AccountDao>,
}

impl EmailLoginReload {
    pub fn new(account_dao: Arc<AccountDao>) -> Self {
        Self { account_dao }
    }
}
#[async_trait]
impl UserLoginReload for EmailLoginReload {
    async fn reload(
        &self,
        session: &SessionBody,
        data: &str,
    ) -> Option<UserAuthResult<(String, u64)>> {
        reload_match_wrap::<EmailLoginMeta>(session, async {
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

pub struct EmailLoginMeta {}
impl AccountLoginMeta for EmailLoginMeta {
    fn login_type() -> String {
        "email".to_string()
    }

    fn login_timeout() -> u64 {
        3 * 24 * 3600
    }
}

pub struct EmailLogin {
    account: Arc<AccountDao>,
    pub email: String,
    pub password: String,
}
impl EmailLogin {
    async fn new_param_valid(email: &str, password: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("login_email"),
                &email,
                &ValidParamCheck::default().add_rule(ValidEmail::default()),
            )
            .add(
                valid_key!("login_password"),
                &password,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(1, 128)),
            )
            .check()?;
        Ok(())
    }
    pub async fn new(account: Arc<AccountDao>, email: &str, password: &str) -> AccountResult<Self> {
        Self::new_param_valid(email, password).await?;
        Ok(Self {
            account,
            email: email.to_string(),
            password: password.to_string(),
        })
    }
}
#[async_trait]
impl AccountLoginParam for EmailLogin {
    type Meta = EmailLoginMeta;
    fn account_name(&self) -> String {
        self.email.clone()
    }
    async fn get_account(&self, _: &AccountLoginEnv) -> AccountResult<(String, AccountModel)> {
        let email = self
            .account
            .account_email
            .find_by_last_email(&self.email)
            .await
            .map_err(auth_user_not_found_map!(self.account_name(), "email"))?;
        email.is_enable()?;

        let user = self
            .account
            .account
            .find_by_id(&email.account_id)
            .await
            .map_err(auth_user_not_found_map!(
                self.account_name(),
                "email [user id]"
            ))?;
        user.is_enable()?;

        let user = auth_check_account_password(&self.account, user, &self.password).await?;
        Ok((email.id.to_string(), user))
    }
}
