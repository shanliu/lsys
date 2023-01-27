use std::{
    error::Error,
    fmt::{Display, Formatter},
    time::SystemTimeError,
};

use lsys_core::now_time;
use lsys_user::dao::account::UserAccountError;
use rand::seq::SliceRandom;
use redis::RedisError;

fn range_client_key() -> String {
    const BASE_STR: &str = "0123456789abcdef0123456789abcdef";
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, 64)
            .cloned()
            .collect(),
    )
    .unwrap_or_else(|_| {
        format!(
            "{:x}",
            md5::compute(now_time().unwrap_or_default().to_string().as_bytes())
        )
    })
}

mod apps;
mod oauth;

pub use apps::*;
pub use oauth::*;

#[derive(Debug)]
pub enum AppsError {
    Sqlx(sqlx::Error),
    System(String),
    Redis(RedisError),
}
impl Display for AppsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for AppsError {}

impl From<sqlx::Error> for AppsError {
    fn from(err: sqlx::Error) -> Self {
        AppsError::Sqlx(err)
    }
}
impl From<RedisError> for AppsError {
    fn from(err: RedisError) -> Self {
        AppsError::Redis(err)
    }
}
impl From<SystemTimeError> for AppsError {
    fn from(err: SystemTimeError) -> Self {
        AppsError::System(err.to_string())
    }
}
impl From<serde_json::Error> for AppsError {
    fn from(err: serde_json::Error) -> Self {
        AppsError::System(format!("{:?}", err))
    }
}
impl From<UserAccountError> for AppsError {
    fn from(err: UserAccountError) -> Self {
        AppsError::System(format!("user error {:?}", err))
    }
}

pub type AppsResult<T> = Result<T, AppsError>;
