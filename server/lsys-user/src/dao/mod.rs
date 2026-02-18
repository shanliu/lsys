mod account;
mod auth;
mod auth_account;
mod auth_code;
mod mfa_login;
mod utils;
use std::sync::Arc;

pub use account::*;
pub use auth::*;
pub use auth_account::*;
pub use auth_code::*;

pub use mfa_login::*;
pub struct UserDao {
    pub account_dao: Arc<AccountDao>,
    pub auth_dao: Arc<UserAuthDao>,
    pub auth_account_dao: Arc<AuthAccount>,
    pub auth_code_dao: Arc<AuthCode>,
    pub mfa_login_dao: Arc<MfaLoginDao>,
}

impl UserDao {
    pub fn new(
        account_dao: Arc<AccountDao>,
        auth_dao: Arc<UserAuthDao>,
        auth_account_dao: Arc<AuthAccount>,
        auth_code_dao: Arc<AuthCode>,
        mfa_login_dao: Arc<MfaLoginDao>,
    ) -> Self {
        Self {
            account_dao,
            auth_dao,
            auth_code_dao,
            auth_account_dao,
            mfa_login_dao,
        }
    }

    pub fn log_types() -> Vec<&'static str> {
        use lsys_logger::dao::ChangeLogData;
        vec![
            account::logger::LogAccountAddress::log_type(),
            account::logger::LogAccountEmail::log_type(),
            account::logger::LogAccountExternal::log_type(),
            account::logger::LogAccountInfo::log_type(),
            account::logger::LogAccountMobile::log_type(),
            account::logger::LogAccountName::log_type(),
            account::logger::LogAccount::log_type(),
            account::logger::LogAccountPassWrod::log_type(),
        ]
    }
}
