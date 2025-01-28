//result 定义
use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};

use super::access::AccessUnauthRes;

#[derive(Debug)]
pub enum RbacError {
    Sqlx(sqlx::Error),
    System(FluentMessage),
    Check(Vec<AccessUnauthRes>),
}

impl IntoFluentMessage for RbacError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            RbacError::Sqlx(e) => fluent_message!("sqlx-error", e),
            RbacError::System(err) => err.to_owned(),
            RbacError::Check(_) => {
                fluent_message!("rbac-check-fail")
            }
        }
    }
}

pub type RbacResult<T> = Result<T, RbacError>;

impl From<sqlx::Error> for RbacError {
    fn from(err: sqlx::Error) -> Self {
        RbacError::Sqlx(err)
    }
}
