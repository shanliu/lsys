use super::super::{LoginData, LoginEnv};
use super::auth_check_user_password;
use crate::dao::account::UserAccount;
use crate::dao::account::UserAccountError;
use crate::dao::auth::{LoginParam, LoginType, UserAuthError, UserAuthResult};

use crate::model::{UserEmailModel, UserModel};
use async_trait::async_trait;

use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::Select;
use std::sync::Arc;

pub struct EmailLogin {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmailLoginData(UserEmailModel);

impl EmailLoginData {
    pub async fn reload(&self, db: &Pool<MySql>) -> UserAuthResult<Self> {
        Ok(EmailLoginData(
            Select::type_new::<UserEmailModel>()
                .reload(&self.0, db)
                .await?,
        ))
    }
}

#[async_trait]
impl LoginParam for EmailLogin {
    async fn get_type(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,
    ) -> UserAuthResult<LoginType> {
        Ok(LoginType {
            time_out: 3600 * 24,
            type_name: "email".to_owned(), //, "Email Login"
        })
    }
    async fn get_user(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,
        account: &Arc<UserAccount>,
        _: &LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)> {
        let email = account
            .user_email
            .find_by_last_email(self.email.clone())
            .await
            .map_err(auth_user_not_found_map!(self.show_name(), "email"))?;
        email.is_enable()?;

        let user =
            account
                .user
                .find_by_id(&email.user_id)
                .await
                .map_err(auth_user_not_found_map!(
                    self.show_name(),
                    "email [user id]"
                ))?;
        user.is_enable()?;

        let user = auth_check_user_password(account, user, &self.password).await?;
        Ok((LoginData::Email(EmailLoginData(email)), user))
    }
    fn show_name(&self) -> String {
        self.email.clone()
    }
}
