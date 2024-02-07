use super::super::{LoginData, LoginEnv};
use crate::dao::account::UserAccount;
use crate::dao::account::UserAccountError;
use crate::dao::auth::{LoginParam, LoginType, UserAuthError, UserAuthResult};

use crate::model::{UserEmailModel, UserModel};
use async_trait::async_trait;

use lsys_core::IntoFluentMessage;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::Select;
use std::sync::Arc;

use tracing::warn;

pub struct EmailCodeLogin {
    pub email: String,
    pub code: String,
}

impl EmailCodeLogin {
    impl_auth_valid_code_method!("email-login",{
        email: &String,
    },{
        email.to_owned()
    },5*60);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailCodeLoginData(UserEmailModel);

impl EmailCodeLoginData {
    pub async fn reload(&self, db: &Pool<MySql>) -> UserAuthResult<Self> {
        Ok(EmailCodeLoginData(
            Select::type_new::<UserEmailModel>()
                .reload(&self.0, db)
                .await?,
        ))
    }
}

#[async_trait]
impl LoginParam for EmailCodeLogin {
    async fn get_type(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,
    ) -> UserAuthResult<LoginType> {
        Ok(LoginType {
            time_out: 3600 * 24,
            type_name: "email-code".to_owned(), //"Email code Login"
        })
    }
    async fn get_user(
        &self,
        _db: &Pool<MySql>,
        redis: &deadpool_redis::Pool,
        account: &Arc<UserAccount>,
        _: &LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)> {
        let email = account
            .user_email
            .find_by_last_email(self.email.clone())
            .await
            .map_err(auth_user_not_found_map!(self.show_name(), "email code"))?;
        email.is_enable()?;

        Self::valid_code_check(redis.to_owned(), &self.code, &self.email).await?;

        let user =
            account
                .user
                .find_by_id(&email.user_id)
                .await
                .map_err(auth_user_not_found_map!(
                    self.show_name(),
                    "email code [user id]"
                ))?;
        user.is_enable()?;

        if let Err(err) = Self::valid_code_clear(redis.to_owned(), &self.email).await {
            warn!(
                "login email clear valid[{}] fail:{}",
                self.email,
                err.to_fluent_message().default_format()
            )
        }

        Ok((LoginData::EmailCode(EmailCodeLoginData(email)), user))
    }
    fn show_name(&self) -> String {
        self.email.clone()
    }
}
