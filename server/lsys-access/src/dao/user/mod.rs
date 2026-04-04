mod cache;
mod data;
mod info;
use crate::model::UserModel;
use cache::AccessUserCache;
pub use data::*;
pub use info::*;

use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{Insert, TableMeta, Update};
use lsys_core::{db::utils::FetchField, valid_key};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{now_time, string_clear, StringClear, STRING_CLEAR_FORMAT};
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use serde::Deserialize;
use serde::Serialize;
use sqlx::{MySql, Pool};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use super::{AccessError, AccessResult};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccessUserAppUserKey {
    app_id: u64,
    user_data: String,
}
impl Display for AccessUserAppUserKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::json!(self))
    }
}
impl FromStr for AccessUserAppUserKey {
    type Err = AccessError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str::<AccessUserAppUserKey>(s)?)
    }
}

pub struct AccessUser {
    db: Pool<MySql>,
    pub(crate) user_cache: Arc<LocalCache<u64, UserModel>>,
    pub(crate) app_user_data: Arc<LocalCache<AccessUserAppUserKey, u64>>,
}

impl AccessUser {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
    ) -> Self {
        Self {
            user_cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            app_user_data: Arc::new(LocalCache::new(remote_notify, config)),
            db,
        }
    }
    async fn sync_user_param_valid(
        &self,
        user_data: impl ToString,
        user_nickname: Option<&str>,
        user_account: Option<&str>,
    ) -> AccessResult<(String, Option<String>, Option<String>)> {
        let user_data = string_clear(user_data, StringClear::Option(STRING_CLEAR_FORMAT), None);

        // 先获取字段长度
        let fetch_field = FetchField::new(&self.db);
        let user_data_max = fetch_field.string_max::<UserModel>(&UserModel::USER_DATA).await.len_or(32);
        let nickname_max = fetch_field.string_max::<UserModel>(&UserModel::USER_NICKNAME).await.len_or(32);
        let account_max = fetch_field.string_max::<UserModel>(&UserModel::USER_ACCOUNT).await.len_or(128);

        let mut valid_param = ValidParam::default();
        valid_param.add(
            valid_key!("user_data"),
            &user_data,
            &ValidParamCheck::default()
                .add_rule(ValidStrlen::range(1, user_data_max))
                .add_rule(ValidPattern::Ident),
        );

        let tmp_user_nickname =
            user_nickname.map(|e| string_clear(e, StringClear::Option(STRING_CLEAR_FORMAT), None));
        if let Some(ref tmp_name) = tmp_user_nickname
            && !tmp_name.is_empty() {
                valid_param.add(
                    valid_key!("user_nickname"),
                    &tmp_name.as_str(),
                    &ValidParamCheck::default().add_rule(ValidStrlen::max(nickname_max)),
                );
            }
        let tmp_user_account =
            user_account.map(|e| string_clear(e, StringClear::Option(STRING_CLEAR_FORMAT), None));
        if let Some(ref account) = tmp_user_account {
            valid_param.add(
                valid_key!("user_account"),
                &account.as_str(),
                &ValidParamCheck::default().add_rule(ValidStrlen::max(account_max)),
            );
        }
        valid_param.check()?;
        //valid finish
        Ok((user_data, tmp_user_nickname, tmp_user_account))
    }
    pub async fn sync_user(
        &self,
        app_id: u64,
        user_data: impl ToString,
        user_nickname: Option<&str>,
        user_account: Option<&str>,
    ) -> AccessResult<u64> {
        let (user_data, tmp_user_nickname, tmp_user_account) = self
            .sync_user_param_valid(user_data, user_nickname, user_account)
            .await?;
        let time = now_time()?;

        // Determine user_nickname value
        let user_nickname_val = if let Some(ref tmp_name) = tmp_user_nickname {
            if tmp_name.is_empty() {
                user_data.clone()
            } else {
                tmp_name.clone()
            }
        } else {
            user_data.clone()
        };

        // Build insert
        let mut insert = Insert::<_, UserModel>::new()
            .set(UserModel::APP_ID, app_id)
            .set(UserModel::USER_DATA, &user_data)
            .set(UserModel::CHANGE_TIME, time)
            .set(UserModel::USER_NICKNAME, &user_nickname_val);

        if let Some(ref account) = tmp_user_account {
            insert = insert.set(UserModel::USER_ACCOUNT, account);
        }

        // Build update
        let mut update = Update::<_, UserModel>::new().set(UserModel::CHANGE_TIME, time);

        if tmp_user_nickname
            .as_ref()
            .map(|e| !e.is_empty())
            .unwrap_or(false)
        {
            update = update.set(UserModel::USER_NICKNAME, &user_nickname_val);
        }

        if let Some(ref account) = tmp_user_account {
            update = update.set(UserModel::USER_ACCOUNT, account);
        }

        match insert.execute_update(update, &self.db).await {
            Ok(row) => {
                self.user_cache.clear(&row.last_insert_id()).await;
                if row.last_insert_id() == 0 {
                    let user = sqlx::query_as::<_, UserModel>(&format!(
                        "select * from {} where app_id=? and user_data=?",
                        UserModel::table_name(),
                    ))
                    .bind(app_id)
                    .bind(&user_data)
                    .fetch_one(&self.db)
                    .await?;
                    Ok(user.id)
                } else {
                    Ok(row.last_insert_id())
                }
            }
            Err(err) => Err(err.into()),
        }
    }
    pub fn cache(&'_ self) -> AccessUserCache<'_> {
        AccessUserCache { dao: self }
    }
}
