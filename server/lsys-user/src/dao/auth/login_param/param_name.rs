use crate::dao::account::{UserAccount, UserAccountError};
use crate::dao::auth::LoginData;
use crate::dao::auth::{LoginParam, LoginType, UserAuthError, UserAuthResult};

use crate::model::{UserModel, UserNameModel};
use async_trait::async_trait;


use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::Select;
use std::sync::Arc;

use super::super::LoginEnv;
use super::auth_check_user_password;

pub struct NameLogin {
    pub name: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NameLoginData(UserNameModel);

impl NameLoginData {
    pub async fn reload(&self, db: &Pool<MySql>) -> UserAuthResult<Self> {
        Ok(NameLoginData(
            Select::type_new::<UserNameModel>()
                .reload(&self.0, db)
                .await?,
        ))
    }
}

#[async_trait]
impl LoginParam for NameLogin {
    async fn get_type(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,
    ) -> UserAuthResult<LoginType> {
        Ok(LoginType {
            time_out: 3600 * 24,
            type_name: "name".to_owned(),
        })
    }
    async fn get_user(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,

        account: &Arc<UserAccount>,
        _: &LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)> {
        let name = account
            .user_name
            .find_by_name(self.name.clone())
            .await
            .map_err(auth_user_not_found_map!(self.show_name(), "name"))?;

        let user = account
            .user
            .find_by_id(&name.user_id)
            .await
            .map_err(auth_user_not_found_map!(self.show_name(), "name [user id]"))?;
        user.is_enable()?;
        let user = auth_check_user_password(account, user, &self.password).await?;
        Ok((LoginData::Name(NameLoginData(name)), user))
    }
    fn show_name(&self) -> String {
        self.name.clone()
    }
}
