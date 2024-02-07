// use std::error::Error;
// use std::fmt::{Display, Formatter};

use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage};

#[derive(Debug)]
pub enum UserRbacError {
    NotLogin(FluentMessage),
    Sqlx(sqlx::Error),
    System(FluentMessage),
    Check(Vec<(String, FluentMessage)>),
}
// impl Display for UserRbacError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self)
//     }
// }
// impl Error for UserRbacError {}

impl IntoFluentMessage for UserRbacError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            UserRbacError::Sqlx(e) => fluent_message!("sqlx-error", e),
            UserRbacError::NotLogin(err) => err.to_owned(),
            UserRbacError::System(err) => err.to_owned(),
            UserRbacError::Check(_) => {
                fluent_message!("rbac-check-fail")
                // let mut msg = fluent_message!("rbac-check-fail");
                // msg.data = err
                //     .into_iter()
                //     .map(|e| (e.0, FluentData::Message(e.1)))
                //     .collect::<Vec<(String, FluentData)>>();
                // msg
            }
        }
    }
}

pub type UserRbacResult<T> = Result<T, UserRbacError>;

impl From<sqlx::Error> for UserRbacError {
    fn from(err: sqlx::Error) -> Self {
        UserRbacError::Sqlx(err)
    }
}
