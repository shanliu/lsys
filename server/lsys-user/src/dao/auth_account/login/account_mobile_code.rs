use super::super::AccountLoginEnv;
use super::{reload_match_wrap, AccountLoginMeta, AccountLoginParam};

use crate::dao::account::AccountError;
use crate::dao::auth::UserLoginReload;
use crate::dao::{AccountDao, AccountResult, UserAuthData, UserAuthResult};
use crate::model::{AccountMobileModel, AccountModel};
use async_trait::async_trait;
use lsys_access::dao::SessionBody;
use lsys_core::fluent_message;

use lsys_core::IntoFluentMessage;
use serde_json::{json, Value};

use std::sync::Arc;
use tracing::warn;

impl MobileCodeLogin {
    /// 验证码生成
    pub fn valid_code(redis: deadpool_redis::Pool) -> lsys_core::ValidCode {
        lsys_core::ValidCode::new(redis, "mobile-login", true)
    }
    /// 获取验证码
    pub async fn valid_code_set<T: lsys_core::ValidCodeData>(
        redis: deadpool_redis::Pool,
        valid_code_data: &mut T,
        area_code: &str,
        mobile: &str,
    ) -> lsys_core::ValidCodeResult<(String, usize)> {
        let valid_code = Self::valid_code(redis);
        let code = valid_code
            .set_code(&format!("{}-{}", area_code, mobile), valid_code_data)
            .await?;
        Ok(code)
    }
    /// 验证码构造器
    pub fn valid_code_builder() -> lsys_core::ValidCodeDataRandom {
        lsys_core::ValidCodeDataRandom::new(60, 30)
    }
    /// 检测验证码
    pub async fn valid_code_check(
        redis: deadpool_redis::Pool,
        code: &str,
        area_code: &str,
        mobile: &str,
    ) -> AccountResult<()> {
        Self::valid_code(redis)
            .check_code(&lsys_core::CheckCodeData::new(
                &format!("{}-{}", area_code, mobile),
                code,
            ))
            .await?;
        Ok(())
    }
    pub async fn valid_code_clear(
        redis: deadpool_redis::Pool,
        area_code: &str,
        mobile: &str,
    ) -> AccountResult<()> {
        let mut builder = Self::valid_code_builder();
        Self::valid_code(redis)
            .destroy_code(&format!("{}-{}", area_code, mobile), &mut builder)
            .await?;
        Ok(())
    }
}

pub struct MobileCodeLoginData {
    data: AccountMobileModel,
}
impl MobileCodeLoginData {
    pub fn new(data: AccountMobileModel) -> Self {
        MobileCodeLoginData { data }
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
                let data = account_dao.account_mobile.cache().find_by_id(&id).await?;
                data.is_enable()?;
                Ok(MobileCodeLoginData::new(data))
            }
        }
    }
}

impl std::ops::Deref for MobileCodeLoginData {
    type Target = AccountMobileModel;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub struct MobileCodeLoginReload {
    account_dao: Arc<AccountDao>,
}

impl MobileCodeLoginReload {
    pub fn new(account_dao: Arc<AccountDao>) -> Self {
        Self { account_dao }
    }
}
#[async_trait]
impl UserLoginReload for MobileCodeLoginReload {
    async fn reload(
        &self,
        session: &SessionBody,
        data: &str,
    ) -> Option<UserAuthResult<(String, u64)>> {
        reload_match_wrap::<MobileCodeLoginMeta>(session, async {
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

pub struct MobileCodeLoginMeta {}
impl AccountLoginMeta for MobileCodeLoginMeta {
    fn login_type() -> String {
        "mobile-code".to_string()
    }

    fn login_timeout() -> u64 {
        3 * 24 * 3600
    }
}

pub struct MobileCodeLogin {
    redis: deadpool_redis::Pool,
    account_dao: Arc<AccountDao>,
    pub area_code: String,
    pub mobile: String,
    pub code: String,
}
impl MobileCodeLogin {
    pub fn new(
        redis: deadpool_redis::Pool,
        account_dao: Arc<AccountDao>,
        area_code: &str,
        mobile: &str,
        code: &str,
    ) -> Self {
        Self {
            redis,
            account_dao,
            area_code: area_code.to_string(),
            mobile: mobile.to_string(),
            code: code.to_string(),
        }
    }
}
#[async_trait]
impl AccountLoginParam for MobileCodeLogin {
    type Meta = MobileCodeLoginMeta;
    fn account_name(&self) -> String {
        format!("{}[{}]", self.mobile, self.area_code)
    }
    async fn get_account(&self, _: &AccountLoginEnv) -> AccountResult<(String, AccountModel)> {
        let mobile = self
            .account_dao
            .account_mobile
            .find_by_last_mobile(&self.area_code, &self.mobile)
            .await
            .map_err(auth_user_not_found_map!(self.account_name(), "mobile code"))?;
        mobile.is_enable()?;

        Self::valid_code_check(
            self.redis.clone(),
            &self.code,
            &self.area_code,
            &self.mobile,
        )
        .await?;

        let user = self
            .account_dao
            .account
            .find_by_id(&mobile.account_id)
            .await
            // .and_then(auth_user_status_and_then!(
            //     user_di.fluent(),
            //     self.show_name().to_owned(),
            //     "mobile code"
            // ))
            .map_err(auth_user_not_found_map!(
                self.account_name(),
                "mobile code [user id]"
            ))?;
        user.is_enable()?;

        if let Err(err) =
            Self::valid_code_clear(self.redis.to_owned(), &self.area_code, &self.mobile).await
        {
            warn!(
                "login mobile clear valid[{}-{}] fail:{}",
                &self.area_code,
                &self.mobile,
                err.to_fluent_message().default_format()
            )
        }
        Ok((mobile.id.to_string(), user))
    }
}
