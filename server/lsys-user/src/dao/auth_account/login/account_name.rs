use crate::dao::{
    AccountDao, AccountError, AccountResult, UserAuthData, UserAuthResult, UserLoginReload,
};

use crate::model::{AccountModel, AccountNameModel};
use async_trait::async_trait;

use lsys_access::dao::SessionBody;
use lsys_core::{
    fluent_message, valid_key, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};
use serde_json::{json, Value};

use std::sync::Arc;

use super::super::AccountLoginEnv;
use super::{auth_check_account_password, reload_match_wrap, AccountLoginMeta, AccountLoginParam};

pub struct NameLoginData {
    data: AccountNameModel,
}

impl NameLoginData {
    pub fn new(data: AccountNameModel) -> Self {
        NameLoginData { data }
    }
    pub fn to_json(&self) -> Value {
        json!(self.data)
    }
    pub async fn from(account_dao: &AccountDao, auth_data: &UserAuthData) -> AccountResult<Self> {
        match auth_data.user().user_data.parse::<u64>() {
            Err(err) => Err(AccountError::System(fluent_message!(
                "account-bad-session",{
                    "msg":err,
                    "data":auth_data.login_data()
                }
            ))),
            Ok(id) => {
                let data = account_dao
                    .account_name
                    .cache()
                    .find_by_account_id(&id)
                    .await?;
                Ok(NameLoginData::new(data))
            }
        }
    }
}

impl std::ops::Deref for NameLoginData {
    type Target = AccountNameModel;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct NameLoginReload {
    account_dao: Arc<AccountDao>,
}

impl NameLoginReload {
    pub fn new(account_dao: Arc<AccountDao>) -> Self {
        Self { account_dao }
    }
}

#[async_trait]
impl UserLoginReload for NameLoginReload {
    async fn reload(
        &self,
        session: &SessionBody,
        _data: &str,
    ) -> Option<UserAuthResult<(String, u64)>> {
        reload_match_wrap::<NameLoginMeta>(session, async {
            let id =
                session.user().user_data.parse::<u64>().map_err(|err| {
                    AccountError::System(fluent_message!("auth-bad-session", err))
                })?;
            let dat = self
                .account_dao
                .account_name
                .find_by_account_id(&id)
                .await?;
            Ok(dat.id.to_string())
        })
        .await
    }
}

pub struct NameLoginMeta {}
impl AccountLoginMeta for NameLoginMeta {
    fn login_type() -> String {
        "name".to_string()
    }
}
pub struct NameLogin {
    account_dao: Arc<AccountDao>,
    pub name: String,
    pub password: String,
}
impl NameLogin {
    async fn new_param_valid(name: &str, password: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("login_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .add(
                valid_key!("login_password"),
                &password,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(1, 128)),
            )
            .check()?;
        Ok(())
    }
    pub async fn new(
        account_dao: Arc<AccountDao>,
        name: &str,
        password: &str,
    ) -> AccountResult<Self> {
        Self::new_param_valid(name, password).await?;
        Ok(Self {
            account_dao,
            name: name.to_string(),
            password: password.to_string(),
        })
    }
}
#[async_trait]
impl AccountLoginParam for NameLogin {
    type Meta = NameLoginMeta;
    fn account_name(&self) -> String {
        self.name.clone()
    }
    async fn get_account(&self, _: &AccountLoginEnv) -> AccountResult<(String, AccountModel)> {
        let name = self
            .account_dao
            .account_name
            .find_by_name(&self.name)
            .await
            .map_err(auth_user_not_found_map!(self.account_name(), "name"))?;

        let user = self
            .account_dao
            .account
            .find_by_id(&name.account_id)
            .await
            .map_err(auth_user_not_found_map!(
                self.account_name(),
                "name [user id]"
            ))?;
        user.is_enable()?;
        let user = auth_check_account_password(&self.account_dao, user, &self.password).await?;
        Ok(("".to_string(), user))
    }
}
