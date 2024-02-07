// use std::error::Error;




use self::user_index::UserIndex;
use self::user_login::UserLogin;

use super::auth::UserPasswordHash;


use lsys_core::{RemoteNotify};

use lsys_logger::dao::ChangeLogger;
use lsys_setting::dao::{SingleSetting};

use sqlx::{MySql, Pool};
use std::sync::Arc;
use user::User;
use user_address::UserAddress;
use user_email::UserEmail;
use user_external::UserExternal;
use user_info::UserInfo;
use user_mobile::UserMobile;
use user_name::UserName;
use user_password::UserPassword;
#[macro_use]
mod macros;

mod logger;
mod result;
mod user_index;
mod utils;
pub use result::*;
pub use utils::*;
pub mod cache;
pub mod user;
pub mod user_address;
pub mod user_email;
pub mod user_external;
pub mod user_info;
pub mod user_login;
pub mod user_mobile;
pub mod user_name;
pub mod user_password;

pub struct UserAccount {
    pub user: Arc<User>,
    pub user_email: Arc<UserEmail>,
    pub user_external: Arc<UserExternal>,
    pub user_mobile: Arc<UserMobile>,
    pub user_name: Arc<UserName>,
    pub user_info: Arc<UserInfo>,
    pub user_address: Arc<UserAddress>,
    pub user_password: Arc<UserPassword>,
    pub user_login: Arc<UserLogin>,
    pub user_passwrod_hash: Arc<UserPasswordHash>,
}

impl UserAccount {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        // fluent: Arc<FluentBuild>,
        setting: Arc<SingleSetting>,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        let user_index = Arc::from(UserIndex::new(db.clone()));
        let password_hash = Arc::from(UserPasswordHash::default());
        UserAccount {
            user: Arc::from(User::new(
                db.clone(),
                // fluent.clone(),
                remote_notify.clone(),
                user_index.clone(),
                logger.clone(),
            )),
            user_email: Arc::from(UserEmail::new(
                db.clone(),
                redis.clone(),
                // fluent.clone(),
                remote_notify.clone(),
                user_index.clone(),
                logger.clone(),
            )),
            user_external: Arc::from(UserExternal::new(
                db.clone(),
                // fluent.clone(),
                remote_notify.clone(),
                user_index.clone(),
                logger.clone(),
            )),
            user_mobile: Arc::from(UserMobile::new(
                db.clone(),
                redis.clone(),
                // fluent.clone(),
                remote_notify.clone(),
                user_index.clone(),
                logger.clone(),
            )),
            user_name: Arc::from(UserName::new(
                db.clone(),
                remote_notify.clone(),
                // fluent.clone(),
                user_index.clone(),
                logger.clone(),
            )),
            user_info: Arc::from(UserInfo::new(
                db.clone(),
                remote_notify.clone(),
                user_index.clone(),
                logger.clone(),
            )),
            user_address: Arc::from(UserAddress::new(
                db.clone(),
                // fluent.clone(),
                remote_notify,
                user_index,
                logger,
            )),
            user_password: Arc::from(UserPassword::new(
                db.clone(),
                setting,
                // fluent,
                redis,
                password_hash.clone(),
            )),
            user_passwrod_hash: password_hash,
            user_login: Arc::from(UserLogin::new(db)),
        }
    }
}
