use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use lsys_core::cache::{LocalCache, LocalCacheClearItem};

use crate::model::{
    AccountAddressModel, AccountEmailModel, AccountExternalModel, AccountInfoModel,
    AccountMobileModel, AccountModel, AccountNameModel,
};

use super::AccountDao;


pub enum AccountLocalCacheClear {
    Account(Arc<LocalCache<u64, AccountModel>>),
    Address(Arc<LocalCache<u64, Vec<AccountAddressModel>>>),
    Email(Arc<LocalCache<u64, AccountEmailModel>>),
    UserEmail(Arc<LocalCache<u64, Vec<u64>>>),
    External(Arc<LocalCache<u64, AccountExternalModel>>),
    UserExternal(Arc<LocalCache<u64, Vec<u64>>>),
    Info(Arc<LocalCache<u64, AccountInfoModel>>),
    Mobile(Arc<LocalCache<u64, AccountMobileModel>>),
    UserMobile(Arc<LocalCache<u64, Vec<u64>>>),
    Name(Arc<LocalCache<u64, AccountNameModel>>),
}

impl AccountLocalCacheClear {
    pub fn new_clears(account: &AccountDao) -> Vec<Self> {
        vec![
            Self::Account(account.account.cache.clone()),
            Self::Address(account.account_address.cache.clone()),
            Self::Email(account.account_email.cache.clone()),
            Self::UserEmail(account.account_email.account_cache.clone()),
            Self::External(account.account_external.cache.clone()),
            Self::UserExternal(account.account_email.account_cache.clone()),
            Self::Info(account.account_info.cache.clone()),
            Self::Mobile(account.account_mobile.cache.clone()),
            Self::UserMobile(account.account_email.account_cache.clone()),
            Self::Name(account.account_name.cache.clone()),
        ]
    }
}

#[async_trait]
impl LocalCacheClearItem for AccountLocalCacheClear {
    fn cache_name(&self) -> &str {
        match self {
            Self::Account(cache) => cache.config().cache_name,
            Self::Address(cache) => cache.config().cache_name,
            Self::Email(cache) => cache.config().cache_name,
            Self::External(cache) => cache.config().cache_name,
            Self::Info(cache) => cache.config().cache_name,
            Self::Mobile(cache) => cache.config().cache_name,
            Self::Name(cache) => cache.config().cache_name,
            Self::UserMobile(cache) => cache.config().cache_name,
            Self::UserExternal(cache) => cache.config().cache_name,
            Self::UserEmail(cache) => cache.config().cache_name,
        }
    }
    async fn clear_from_message(&self, msg: &str) -> Result<(), String> {
        let key = &u64::from_str(msg).map_err(|e| e.to_string())?;
        match self {
            Self::Account(cache) => cache.del(key).await,
            Self::Address(cache) => cache.del(key).await,
            Self::Email(cache) => cache.del(key).await,
            Self::External(cache) => cache.del(key).await,
            Self::Info(cache) => cache.del(key).await,
            Self::Mobile(cache) => cache.del(key).await,
            Self::Name(cache) => cache.del(key).await,
            Self::UserMobile(cache) => cache.del(key).await,
            Self::UserExternal(cache) =>cache.del(key).await,
            Self::UserEmail(cache) => cache.del(key).await,
        };
        Ok(())
    }
}
