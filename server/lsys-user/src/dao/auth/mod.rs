mod login;
mod login_param;
mod login_store;
mod session;
use deadpool_redis::PoolError;

pub use login::*;
pub use login_store::*;
pub use session::*;

use crate::dao::account::UserAccountError;
use lsys_core::{fluent_message, FluentMessage, ValidCodeError};

use redis::RedisError;

use std::error::Error;
use std::fmt::{Display, Formatter};

use std::string::FromUtf8Error;

use std::time::SystemTimeError;

pub use self::login_param::*;

//统一错误
#[derive(Debug)]
pub enum UserAuthError {
    TokenParse(FluentMessage),
    Sqlx(sqlx::Error),
    Redis(RedisError),
    RedisPool(PoolError),
    PasswordNotMatch((u64, FluentMessage)),
    PasswordNotSet((u64, FluentMessage)),
    StatusError((u64, FluentMessage)),
    ValidCode(ValidCodeError),
    UserNotFind(FluentMessage),
    NotLogin(FluentMessage),
    UserAccount(UserAccountError),
    System(FluentMessage),
    CheckUserLock((u64, FluentMessage)),
    CheckCaptchaNeed(FluentMessage),
}
impl Display for UserAuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for UserAuthError {}

pub type UserAuthResult<T> = Result<T, UserAuthError>;

impl From<sqlx::Error> for UserAuthError {
    fn from(err: sqlx::Error) -> Self {
        UserAuthError::Sqlx(err)
    }
}
impl From<SystemTimeError> for UserAuthError {
    fn from(err: SystemTimeError) -> Self {
        UserAuthError::System(err.to_string().into())
    }
}
impl From<RedisError> for UserAuthError {
    fn from(err: RedisError) -> Self {
        UserAuthError::Redis(err)
    }
}
impl From<PoolError> for UserAuthError {
    fn from(err: PoolError) -> Self {
        UserAuthError::RedisPool(err)
    }
}
impl From<serde_json::Error> for UserAuthError {
    fn from(err: serde_json::Error) -> Self {
        UserAuthError::System(fluent_message!("serde-error", err))
    }
}
impl From<UserAccountError> for UserAuthError {
    fn from(err: UserAccountError) -> Self {
        UserAuthError::UserAccount(err)
    }
}
impl From<FromUtf8Error> for UserAuthError {
    fn from(err: FromUtf8Error) -> Self {
        UserAuthError::System(fluent_message!("utf-error", err))
    }
}
impl From<ValidCodeError> for UserAuthError {
    fn from(err: ValidCodeError) -> Self {
        UserAuthError::ValidCode(err)
    }
}
