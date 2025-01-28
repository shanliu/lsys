use crate::dao::account::AccountDao;
use crate::dao::account::AccountError;
use crate::dao::AccountResult;
use crate::dao::UserAuthData;
use crate::dao::UserAuthResult;
use crate::dao::UserLoginReload;

use super::super::AccountLoginEnv;
use super::reload_match_wrap;
use super::AccountLoginMeta;
use super::AccountLoginParam;
use crate::model::{AccountExternalModel, AccountModel};
use async_trait::async_trait;
use lsys_access::dao::SessionBody;
use lsys_core::fluent_message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;
use std::sync::Arc;

pub struct ExternalLoginData {
    data: AccountExternalModel,
    ext_data: String,
}
impl ExternalLoginData {
    pub fn new(data: AccountExternalModel, ext_data: &str) -> Self {
        Self {
            data,
            ext_data: ext_data.to_string(),
        }
    }
    pub fn parse_ext_data<'de, T>(&'de self) -> serde_json::Result<T>
    where
        T: Deserialize<'de> + Send + Sync,
    {
        let str = self.ext_data.as_str();
        serde_json::from_str::<T>(str)
    }
    pub fn to_json(&self) -> Value {
        json!(self.data)
    }
    pub async fn from(account_dao: &AccountDao, auth_data: &UserAuthData) -> AccountResult<Self> {
        let mut tmp = auth_data.login_data().split(',');
        match tmp.next() {
            Some(e) => match e.parse::<u64>() {
                Err(err) => Err(AccountError::System(fluent_message!(
                    "account-bad-session",
                    err
                ))),
                Ok(id) => {
                    let data = account_dao.account_external.cache().find_by_id(&id).await?;
                    Ok(ExternalLoginData::new(
                        data,
                        &tmp.collect::<Vec<&str>>().join(""),
                    ))
                }
            },
            None => Err(AccountError::System(fluent_message!(
                "account-bad-session",
                "bad string"
            ))),
        }
    }
}

impl std::ops::Deref for ExternalLoginData {
    type Target = AccountExternalModel;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct ExternalLoginReload {
    account_dao: Arc<AccountDao>,
}

impl ExternalLoginReload {
    pub fn new(account_dao: Arc<AccountDao>) -> Self {
        Self { account_dao }
    }
}
#[async_trait]
impl UserLoginReload for ExternalLoginReload {
    async fn reload(
        &self,
        session: &SessionBody,
        data: &str,
    ) -> Option<UserAuthResult<(String, u64)>> {
        reload_match_wrap::<ExternalLoginMeta>(session, async {
            let mut tmp = data.split(',');
            match tmp.next() {
                Some(e) => {
                    let id = e.parse::<u64>().map_err(|err| {
                        AccountError::System(fluent_message!("auth-bad-session", err))
                    })?;
                    let dat = self.account_dao.account_external.find_by_id(&id).await?;
                    dat.is_enable()?;
                    Ok(format!(
                        "{},{}",
                        dat.id,
                        tmp.collect::<Vec<&str>>().join("")
                    ))
                }
                None => Err(AccountError::System(fluent_message!(
                    "account-bad-session",
                    "bad string"
                ))),
            }
        })
        .await
    }
}

pub struct ExternalLoginMeta {}
impl AccountLoginMeta for ExternalLoginMeta {
    fn login_type() -> String {
        "external".to_string()
    }
}

pub struct ExternalLogin<T: Serialize + Send + Sync> {
    account_dao: Arc<AccountDao>,
    pub external: AccountExternalModel,
    pub ext_data: T,
}
impl<T: Serialize + Send + Sync> ExternalLogin<T> {
    pub fn new(account_dao: Arc<AccountDao>, external: AccountExternalModel, ext_data: T) -> Self {
        Self {
            account_dao,
            external,
            ext_data,
        }
    }
}
#[async_trait]
impl<T: Serialize + Send + Sync> AccountLoginParam for ExternalLogin<T> {
    type Meta = ExternalLoginMeta;
    fn account_name(&self) -> String {
        self.external.external_id.clone()
    }
    async fn get_account(&self, _: &AccountLoginEnv) -> AccountResult<(String, AccountModel)> {
        self.external.is_enable()?;
        let ext_data = serde_json::to_string(&self.ext_data)?;
        let user = self
            .account_dao
            .account
            .find_by_id(&self.external.account_id)
            .await
            .map_err(auth_user_not_found_map!(
                self.account_name(),
                "external account [user id]"
            ))?;
        user.is_enable()?;

        Ok((format!("{},{}", self.external.id, ext_data), user))
    }
}
