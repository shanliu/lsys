// 内部账号实现
use lsys_access::dao::{AccessDao, SessionBody};
use lsys_core::{cache::LocalCacheConfig, fluent_message, IntoFluentMessage, RemoteNotify};

use lsys_logger::dao::ChangeLoggerDao;

use lsys_setting::dao::SingleSetting;
use sqlx::{MySql, Pool};
use std::sync::Arc;

#[allow(clippy::module_inception)]
mod account;
mod account_address;
mod account_email;
mod account_external;
mod account_index;
mod account_info;
mod account_login_history;
mod account_mobile;
mod account_name;
mod account_password;
mod cache;
mod logger;
mod password;
mod result;
mod utils;

use account::*;
use account_address::*;
use account_email::*;
use account_external::*;
use account_index::*;
use account_info::*;
pub use account_login_history::*;
use account_mobile::*;
use account_name::*;
use account_password::*;
pub use cache::*;
pub use password::*;
pub use result::*;
pub use utils::*;

use crate::model::AccountModel;

use super::{UserAuthError, UserAuthResult};

pub struct AccountConfig {
    pub account_cache: LocalCacheConfig,
    pub email_cache: LocalCacheConfig,
    pub mobile_cache: LocalCacheConfig,
    pub name_cache: LocalCacheConfig,
    pub info_cache: LocalCacheConfig,
    pub address_cache: LocalCacheConfig,
    pub external_cache: LocalCacheConfig,
}

impl AccountConfig {
    pub fn new(use_cache: bool) -> Self {
        Self {
            account_cache: LocalCacheConfig::new(
                "account",
                if use_cache { None } else { Some(0) },
                None,
            ),
            email_cache: LocalCacheConfig::new(
                "account-email",
                if use_cache { None } else { Some(0) },
                None,
            ),
            mobile_cache: LocalCacheConfig::new(
                "account-mobile",
                if use_cache { None } else { Some(0) },
                None,
            ),
            name_cache: LocalCacheConfig::new(
                "account-name",
                if use_cache { None } else { Some(0) },
                None,
            ),
            info_cache: LocalCacheConfig::new(
                "account-info",
                if use_cache { None } else { Some(0) },
                None,
            ),
            address_cache: LocalCacheConfig::new(
                "account-address",
                if use_cache { None } else { Some(0) },
                None,
            ),
            external_cache: LocalCacheConfig::new(
                "account-external",
                if use_cache { None } else { Some(0) },
                None,
            ),
        }
    }
}

pub struct AccountDao {
    pub account: Arc<Account>,
    pub account_email: Arc<AccountEmail>,
    pub account_external: Arc<AccountExternal>,
    pub account_mobile: Arc<AccountMobile>,
    pub account_name: Arc<AccountName>,
    pub account_info: Arc<AccountInfo>,
    pub account_address: Arc<AccountAddress>,
    pub account_password: Arc<AccountPassword>,
    pub account_login_hostory: Arc<AccountLoginHistory>,
    pub account_passwrod_hash: Arc<AccountPasswordHash>,
}

impl AccountDao {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        setting: Arc<SingleSetting>,
        access: Arc<AccessDao>,
        config: AccountConfig,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        let account_index = Arc::from(AccountIndex::new(db.clone()));
        let password_hash = Arc::from(AccountPasswordHash::default());
        let account = Arc::from(Account::new(
            db.clone(),
            access.clone(),
            account_index.clone(),
            remote_notify.clone(),
            config.account_cache,
            logger.clone(),
        ));
        let account_login_hostory = Arc::from(AccountLoginHistory::new(db.clone()));
        AccountDao {
            account,
            account_email: Arc::from(AccountEmail::new(
                db.clone(),
                redis.clone(),
                account_index.clone(),
                remote_notify.clone(),
                config.email_cache,
                logger.clone(),
            )),
            account_external: Arc::from(AccountExternal::new(
                db.clone(),
                account_index.clone(),
                remote_notify.clone(),
                config.external_cache,
                logger.clone(),
            )),
            account_mobile: Arc::from(AccountMobile::new(
                db.clone(),
                redis.clone(),
                account_index.clone(),
                remote_notify.clone(),
                config.mobile_cache,
                logger.clone(),
            )),
            account_name: Arc::from(AccountName::new(
                db.clone(),
                account_index.clone(),
                access,
                remote_notify.clone(),
                config.name_cache,
                logger.clone(),
            )),
            account_info: Arc::from(AccountInfo::new(
                db.clone(),
                account_index.clone(),
                remote_notify.clone(),
                config.info_cache,
                logger.clone(),
            )),
            account_address: Arc::from(AccountAddress::new(
                db.clone(),
                account_index,
                remote_notify.clone(),
                config.address_cache,
                logger.clone(),
            )),
            account_password: Arc::from(AccountPassword::new(
                db,
                setting,
                // fluent,
                redis,
                logger,
                password_hash.clone(),
            )),
            account_passwrod_hash: password_hash,
            account_login_hostory,
        }
    }
    pub async fn session_account(
        &self,
        session_body: &SessionBody,
    ) -> UserAuthResult<AccountModel> {
        self.account
            .cache()
            .find_by_id(&session_body.account_id()?)
            .await
            .map_err(|e| {
                UserAuthError::System(fluent_message!("auth-find-error", e.to_fluent_message()))
            })
    }
}
