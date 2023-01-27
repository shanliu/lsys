use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use lsys_core::cache::{LocalCache, LocalCacheClearItem};

use crate::model::{
    UserAddressModel, UserEmailModel, UserExternalModel, UserInfoModel, UserMobileModel, UserModel,
    UserNameModel,
};

use super::UserAccount;

pub enum UserAccountLocalCacheClear {
    User(Arc<LocalCache<u64, UserModel>>),
    Address(Arc<LocalCache<u64, Vec<UserAddressModel>>>),
    Email(Arc<LocalCache<u64, Vec<UserEmailModel>>>),
    External(Arc<LocalCache<u64, Vec<UserExternalModel>>>),
    Info(Arc<LocalCache<u64, UserInfoModel>>),
    Mobile(Arc<LocalCache<u64, Vec<UserMobileModel>>>),
    Name(Arc<LocalCache<u64, UserNameModel>>),
}

impl UserAccountLocalCacheClear {
    pub fn new_clears(account: &UserAccount) -> Vec<Self> {
        vec![
            Self::User(account.user.cache.clone()),
            Self::Address(account.user_address.cache.clone()),
            Self::Email(account.user_email.cache.clone()),
            Self::External(account.user_external.cache.clone()),
            Self::Info(account.user_info.cache.clone()),
            Self::Mobile(account.user_mobile.cache.clone()),
            Self::Name(account.user_name.cache.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem for UserAccountLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            Self::User(cache) => cache.config().cache_name,
            Self::Address(cache) => cache.config().cache_name,
            Self::Email(cache) => cache.config().cache_name,
            Self::External(cache) => cache.config().cache_name,
            Self::Info(cache) => cache.config().cache_name,
            Self::Mobile(cache) => cache.config().cache_name,
            Self::Name(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        let key = &u64::from_str(msg).map_err(|e| e.to_string())?;
        match self {
            Self::User(cache) => cache.del(key).await,
            Self::Address(cache) => cache.del(key).await,
            Self::Email(cache) => cache.del(key).await,
            Self::External(cache) => cache.del(key).await,
            Self::Info(cache) => cache.del(key).await,
            Self::Mobile(cache) => cache.del(key).await,
            Self::Name(cache) => cache.del(key).await,
        };
        Ok(())
    }
}
