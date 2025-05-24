use crate::dao::account::AccountError;
use crate::dao::auth::UserLoginReload;

use super::super::AccountLoginEnv;
use super::{auth_check_account_password, reload_match_wrap, AccountLoginMeta, AccountLoginParam};
use crate::dao::{AccountDao, AccountResult, UserAuthData, UserAuthResult};
use crate::model::{AccountMobileModel, AccountModel};
use async_trait::async_trait;
use lsys_access::dao::SessionBody;
use lsys_core::{
    fluent_message, valid_key, ValidMobile, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};
use serde_json::{json, Value};

use std::sync::Arc;

pub struct MobileLoginData {
    data: AccountMobileModel,
}

impl MobileLoginData {
    pub fn new(data: AccountMobileModel) -> Self {
        MobileLoginData { data }
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
                let data = account_dao.account_mobile.cache().find_by_id(&id).await?;
                data.is_enable()?;
                Ok(MobileLoginData::new(data))
            }
        }
    }
}

impl std::ops::Deref for MobileLoginData {
    type Target = AccountMobileModel;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct MobileLoginReload {
    account_dao: Arc<AccountDao>,
}

impl MobileLoginReload {
    pub fn new(account_dao: Arc<AccountDao>) -> Self {
        Self { account_dao }
    }
}

#[async_trait]
impl UserLoginReload for MobileLoginReload {
    async fn reload(
        &self,
        session: &SessionBody,
        data: &str,
    ) -> Option<UserAuthResult<(String, u64)>> {
        reload_match_wrap::<MobileLoginMeta>(session, async {
            let id = data
                .parse::<u64>()
                .map_err(|err| AccountError::System(fluent_message!("auth-bad-session", err)))?;
            let dat = self.account_dao.account_mobile.find_by_id(&id).await?;
            dat.is_enable()?;
            Ok(dat.id.to_string())
        })
        .await
    }
}
pub struct MobileLoginMeta {}
impl AccountLoginMeta for MobileLoginMeta {
    fn login_type() -> String {
        "mobile-code".to_string()
    }
}
pub struct MobileLogin {
    account_dao: Arc<AccountDao>,
    pub area_code: String,
    pub mobile: String,
    pub password: String,
}
impl MobileLogin {
    async fn new_param_valid(area_code: &str, mobile: &str, password: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("login_area_code"),
                &area_code,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Numeric)
                    .add_rule(ValidStrlen::range(2, 6)),
            )
            .add(
                valid_key!("login_mobile"),
                &mobile,
                &ValidParamCheck::default().add_rule(ValidMobile::default()),
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
        area_code: &str,
        mobile: &str,
        password: &str,
    ) -> AccountResult<Self> {
        Self::new_param_valid(area_code, mobile, password).await?;
        Ok(Self {
            account_dao,
            area_code: area_code.to_string(),
            mobile: mobile.to_string(),
            password: password.to_string(),
        })
    }
}
#[async_trait]
impl AccountLoginParam for MobileLogin {
    type Meta = MobileLoginMeta;
    fn account_name(&self) -> String {
        format!("{}-{}", self.area_code, self.mobile,)
    }
    async fn get_account(&self, _: &AccountLoginEnv) -> AccountResult<(String, AccountModel)> {
        let mobile = self
            .account_dao
            .account_mobile
            .find_by_last_mobile(&self.area_code, &self.mobile)
            .await
            .map_err(auth_user_not_found_map!(self.account_name(), "mobile"))?;
        mobile.is_enable()?;
        let user = self
            .account_dao
            .account
            .find_by_id(&mobile.account_id)
            .await
            .map_err(auth_user_not_found_map!(
                self.account_name(),
                "mobile [user id]"
            ))?;
        user.is_enable()?;
        let user = auth_check_account_password(&self.account_dao, user, &self.password).await?;
        Ok((mobile.id.to_string(), user))
    }
}
