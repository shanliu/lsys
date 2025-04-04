
// use std::fmt::Display;
// use std::fmt::Formatter;

use lsys_core::fluent_message;
use lsys_core::FluentMessage;
use lsys_core::IntoFluentMessage;

#[derive(Debug)]
pub enum GitDocError {
    Sqlx(sqlx::Error),
    Git(git2::Error),
    System(FluentMessage),
    Remote(FluentMessage),
}

impl IntoFluentMessage for GitDocError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            GitDocError::Sqlx(e) => fluent_message!("sqlx-error", e),
            GitDocError::Git(e) => fluent_message!("git-error", e),
            GitDocError::System(e) => e.to_owned(),
            GitDocError::Remote(e) => e.to_owned(),
        }
    }
}

impl From<git2::Error> for GitDocError {
    fn from(err: git2::Error) -> Self {
        GitDocError::Git(err)
    }
}
impl From<sqlx::Error> for GitDocError {
    fn from(err: sqlx::Error) -> Self {
        GitDocError::Sqlx(err)
    }
}

pub type GitDocResult<T> = Result<T, GitDocError>;

pub(crate) enum CloneError {
    VerNotMatch(String),
    Err(String),
}
pub(crate) type CloneResult = Result<(), CloneError>;
