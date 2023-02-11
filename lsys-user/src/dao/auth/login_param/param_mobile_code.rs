use super::super::{LoginData, LoginEnv};
use crate::dao::account::UserAccount;
use crate::dao::account::UserAccountError;
use crate::dao::auth::{LoginParam, LoginType, UserAuthError, UserAuthResult};

use crate::model::{UserMobileModel, UserModel};
use async_trait::async_trait;
use lsys_core::get_message;
use lsys_core::FluentMessage;

use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::Select;
use std::sync::Arc;
use tracing::warn;
pub struct MobileCodeLogin {
    pub area_code: String,
    pub mobile: String,
    pub code: String,
}

impl MobileCodeLogin {
    impl_auth_valid_code_method!("mobile-login",{
        area_code: &String,
        mobile: &String,
    },{
       format!("{}{}",area_code,mobile)
    },60);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobileCodeLoginData(UserMobileModel);

impl MobileCodeLoginData {
    pub async fn reload(&self, db: &Pool<MySql>) -> UserAuthResult<Self> {
        Ok(MobileCodeLoginData(
            Select::type_new::<UserMobileModel>()
                .reload(&self.0, db)
                .await?,
        ))
    }
}

#[async_trait]
impl LoginParam for MobileCodeLogin {
    async fn get_type(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,
        fluent: &Arc<FluentMessage>,
    ) -> UserAuthResult<LoginType> {
        Ok(LoginType {
            time_out: 3600 * 24,
            type_name: get_message!(fluent, "auth-login-type-mobile", "Mobile Code Login"),
        })
    }
    async fn get_user(
        &self,
        _db: &Pool<MySql>,
        redis: &deadpool_redis::Pool,
        fluent: &Arc<FluentMessage>,
        account: &Arc<UserAccount>,
        _: &LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)> {
        let mobile = account
            .user_mobile
            .find_by_last_mobile(self.area_code.clone(), self.mobile.clone())
            .await
            .map_err(auth_user_not_found_map!(
                fluent,
                self.show_name(),
                "mobile code"
            ))?;
        mobile.is_enable()?;

        Self::valid_code_check(redis.clone(), &self.code, &self.area_code, &self.mobile).await?;

        let user = account
            .user
            .find_by_id(&mobile.user_id)
            .await
            // .and_then(auth_user_status_and_then!(
            //     user_di.fluent(),
            //     self.show_name().to_owned(),
            //     "mobile code"
            // ))
            .map_err(auth_user_not_found_map!(
                fluent,
                self.show_name(),
                "mobile code [user id]"
            ))?;
        user.is_enable()?;

        if let Err(err) =
            Self::valid_code_clear(redis.to_owned(), &self.area_code, &self.mobile).await
        {
            warn!(
                "login mobile clear valid[{}-{}] fail:{}",
                &self.area_code, &self.mobile, err
            )
        }

        Ok((LoginData::MobileCode(MobileCodeLoginData(mobile)), user))
    }
    fn show_name(&self) -> String {
        format!("{}[{}]", self.mobile, self.area_code)
    }
}
