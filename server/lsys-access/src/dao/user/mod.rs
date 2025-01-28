mod cache;
mod data;
mod info;
use crate::model::{UserModel, UserModelRef};
use cache::AccessUserCache;
pub use info::*;

use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{Insert, Update};
use lsys_core::{fluent_message, now_time, RemoteNotify};
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
    pub async fn sync_user(
        &self,
        app_id: u64,
        user_data: impl ToString,
        user_name: Option<&str>,
        user_account: Option<&str>,
    ) -> AccessResult<u64> {
        let time = now_time()?;
        let user_data = user_data.to_string();
        if user_data.trim().is_empty() {
            return Err(AccessError::System(fluent_message!("access-bad-user",{
                "user_data":user_data,
            })));
        }
        let mut vdata = lsys_core::model_option_set!(UserModelRef,{
            app_id:app_id,
            user_data:user_data,
            change_time:time,
        });
        let tmp_user_name = user_name.map(|e| e.to_string());
        if tmp_user_name
            .as_ref()
            .map(|e| !e.is_empty())
            .unwrap_or(false)
        {
            vdata.user_name = tmp_user_name.as_ref();
        } else {
            vdata.user_name = Some(&user_data);
        }
        let tmp_user_account = user_account.map(|e| e.to_string());
        vdata.user_account = tmp_user_account.as_ref();
        let mut change = lsys_core::model_option_set!(UserModelRef,{
            change_time:time,
        });
        if tmp_user_name
            .as_ref()
            .map(|e| !e.is_empty())
            .unwrap_or(false)
        {
            change.user_name = tmp_user_name.as_ref();
        }
        change.user_account = tmp_user_account.as_ref();
        match Insert::<sqlx::MySql, UserModel, _>::new(vdata)
            .execute_update(&Update::<MySql, UserModel, _>::new(change), &self.db)
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
