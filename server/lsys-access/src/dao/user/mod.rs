mod cache;
mod data;
mod info;
use crate::model::{UserModel, UserModelRef};
use cache::AccessUserCache;
pub use data::*;
pub use info::*;

use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{Insert, Update};
use lsys_core::{
    now_time, string_clear, valid_key, RemoteNotify, StringClear, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen, STRING_CLEAR_FORMAT,
};
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
        let mut valid_param = ValidParam::default();
        valid_param.add(
            valid_key!("user_data"),
            &user_data,
            &ValidParamCheck::default()
                .add_rule(ValidStrlen::range(1, 32))
                .add_rule(ValidPattern::Ident),
        );

        let tmp_user_nickname =
            user_nickname.map(|e| string_clear(e, StringClear::Option(STRING_CLEAR_FORMAT), None));
        if let Some(ref tmp_name) = tmp_user_nickname {
            if !tmp_name.is_empty() {
                valid_param.add(
                    valid_key!("user_nickname"),
                    &tmp_name.as_str(),
                    &ValidParamCheck::default().add_rule(ValidStrlen::max(32)),
                );
            }
        }
        let tmp_user_account =
            user_account.map(|e| string_clear(e, StringClear::Option(STRING_CLEAR_FORMAT), None));
        if let Some(ref account) = tmp_user_account {
            valid_param.add(
                valid_key!("user_account"),
                &account.as_str(),
                &ValidParamCheck::default().add_rule(ValidStrlen::max(128)),
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
        let mut vdata = lsys_core::model_option_set!(UserModelRef,{
            app_id:app_id,
            user_data:user_data,
            change_time:time,
        });
        if let Some(ref tmp_name) = tmp_user_nickname {
            if tmp_name.is_empty() {
                vdata.user_nickname = Some(&user_data);
            } else {
                vdata.user_nickname = tmp_user_nickname.as_ref();
            }
        } else {
            vdata.user_nickname = Some(&user_data);
        }
        vdata.user_account = tmp_user_account.as_ref();
        let mut change = lsys_core::model_option_set!(UserModelRef,{
            change_time:time,
        });
        if tmp_user_nickname
            .as_ref()
            .map(|e| !e.is_empty())
            .unwrap_or(false)
        {
            change.user_nickname = tmp_user_nickname.as_ref();
        }
        change.user_account = tmp_user_account.as_ref();
        match Insert::<UserModel, _>::new(vdata)
            .execute_update(&Update::<UserModel, _>::new(change), &self.db)
            .await
        {
            Ok(row) => {
                self.user_cache.clear(&row.last_insert_id()).await;
                Ok(row.last_insert_id())
            }
            Err(err) => Err(err.into()),
        }
    }
    pub fn cache(&'_ self) -> AccessUserCache<'_> {
        AccessUserCache { dao: self }
    }
}
