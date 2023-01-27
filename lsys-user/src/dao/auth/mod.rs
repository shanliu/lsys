mod login;
mod login_param;
mod login_store;
mod session;
pub use login::*;
pub use login_store::*;
pub use session::*;

use crate::dao::account::UserAccountError;
use lsys_core::ValidCodeError;

use redis::RedisError;

use std::error::Error;
use std::fmt::{Display, Formatter};

use std::string::FromUtf8Error;

use std::time::SystemTimeError;

pub use self::login_param::*;

//统一错误
#[derive(Debug)]
pub enum UserAuthError {
    TokenParse(String),
    PasswordNotMatch((u64, String)),
    PasswordNotSet((u64, String)),
    StatusError((u64, String)),
    ValidCode(ValidCodeError),
    UserNotFind(String),
    NotLogin(String),
    Sqlx(sqlx::Error),
    UserAccount(UserAccountError),
    System(String),
    CheckUserLock(u64),
    CheckCaptchaNeed(String),
    Redis(RedisError),
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
        UserAuthError::System(err.to_string())
    }
}
impl From<RedisError> for UserAuthError {
    fn from(err: RedisError) -> Self {
        UserAuthError::Redis(err)
    }
}
impl From<serde_json::Error> for UserAuthError {
    fn from(err: serde_json::Error) -> Self {
        UserAuthError::System(format!("{:?}", err))
    }
}
impl From<UserAccountError> for UserAuthError {
    fn from(err: UserAccountError) -> Self {
        if let UserAccountError::Status((uid, err)) = err {
            return UserAuthError::StatusError((uid, err));
        }
        if let UserAccountError::Sqlx(err) = err {
            return UserAuthError::Sqlx(err);
        }
        if let UserAccountError::Redis(err) = err {
            return UserAuthError::Redis(err);
        }
        if let UserAccountError::ValidCode(err) = err {
            return UserAuthError::ValidCode(err);
        }
        UserAuthError::UserAccount(err)
    }
}
impl From<FromUtf8Error> for UserAuthError {
    fn from(err: FromUtf8Error) -> Self {
        UserAuthError::System(format!("{:?}", err))
    }
}
impl From<ValidCodeError> for UserAuthError {
    fn from(err: ValidCodeError) -> Self {
        UserAuthError::ValidCode(err)
    }
}
