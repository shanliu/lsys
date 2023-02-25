use crate::dao::account::UserAccountError;

use super::super::{LoginData, LoginEnv};
use crate::dao::account::UserAccount;
use crate::dao::auth::{LoginParam, LoginType, UserAuthError, UserAuthResult};
use crate::model::{UserExternalModel, UserModel};
use async_trait::async_trait;
use lsys_core::{get_message, FluentMessage};

use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use sqlx_model::Select;
use std::sync::Arc;
pub struct ExternalLogin<T: Serialize + Send + Sync> {
    pub external: UserExternalModel,
    pub ext_data: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalLoginData(pub UserExternalModel, pub String);

impl ExternalLoginData {
    pub fn parse_ext_data<'de, T>(&'de self) -> serde_json::Result<T>
    where
        T: Deserialize<'de> + Send + Sync,
    {
        let str = self.1.as_str();
        serde_json::from_str::<T>(str)
    }
}

impl ExternalLoginData {
    pub async fn reload(&self, db: &Pool<MySql>) -> UserAuthResult<Self> {
        Ok(ExternalLoginData(
            Select::type_new::<UserExternalModel>()
                .reload(&self.0, db)
                .await?,
            self.1.clone(),
        ))
    }
}

#[async_trait]
impl<T: Serialize + Send + Sync> LoginParam for ExternalLogin<T> {
    async fn get_type(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,
        fluent: &Arc<FluentMessage>,
    ) -> UserAuthResult<LoginType> {
        Ok(LoginType {
            time_out: 3600 * 24,
            type_name: get_message!(fluent, "auth-login-type-external", "External Login"),
        })
    }
    async fn get_user(
        &self,
        _db: &Pool<MySql>,
        _redis: &deadpool_redis::Pool,
        fluent: &Arc<FluentMessage>,
        account: &Arc<UserAccount>,
        _: &LoginEnv,
    ) -> UserAuthResult<(LoginData, UserModel)> {
        self.external.is_enable()?;
        let ext_data = serde_json::to_string(&self.ext_data)?;
        let user = account
            .user
            .find_by_id(&self.external.user_id)
            .await
            .map_err(auth_user_not_found_map!(
                fluent,
                self.show_name(),
                "external account [user id]"
            ))?;
        user.is_enable()?;
        Ok((
            LoginData::External(ExternalLoginData(self.external.clone(), ext_data)),
            user,
        ))
    }
    fn show_name(&self) -> String {
        self.external.external_id.clone()
    }
}
