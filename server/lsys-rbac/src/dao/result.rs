//result 定义
use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidError};

use super::access::AccessUnauthRes;

#[derive(Debug)]
pub enum RbacError {
    Sqlx(sqlx::Error),
    System(FluentMessage),
    Check(Vec<AccessUnauthRes>),
    Vaild(ValidError),
}

impl IntoFluentMessage for RbacError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            RbacError::Sqlx(e) => fluent_message!("sqlx-error", e),
            RbacError::System(err) => err.to_owned(),
            RbacError::Check(_) => {
                fluent_message!("rbac-check-fail")
            }
            RbacError::Vaild(e) => e.to_fluent_message(),
        }
    }
}

pub type RbacResult<T> = Result<T, RbacError>;

impl From<sqlx::Error> for RbacError {
    fn from(err: sqlx::Error) -> Self {
        RbacError::Sqlx(err)
    }
}

impl From<ValidError> for RbacError {
    fn from(err: ValidError) -> Self {
        RbacError::Vaild(err)
    }
}
