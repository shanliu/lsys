use lsys_core::{fluent_message, FluentMessage, IntoFluentMessage, ValidError};

#[derive(Debug)]
pub enum MfaError {
    Sqlx(sqlx::Error),
    ValidParam(ValidError),

    NotEnabled,
    VerifyFailed,
    Replay,

    /// Invalid secret format or decode error.
    SecretInvalid,
    TokenExpired,
}

impl IntoFluentMessage for MfaError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            MfaError::Sqlx(e) => fluent_message!("sqlx-error", e),
            MfaError::ValidParam(e) => e.to_fluent_message(),
            MfaError::NotEnabled => fluent_message!("mfa-not-enabled"),
            MfaError::VerifyFailed => fluent_message!("mfa-verify-failed"),
            MfaError::Replay => fluent_message!("mfa-replay"),
            MfaError::SecretInvalid => fluent_message!("mfa-secret-invalid"),
            MfaError::TokenExpired => fluent_message!("mfa-token-expired"),
        }
    }
}

impl From<sqlx::Error> for MfaError {
    fn from(err: sqlx::Error) -> Self {
        MfaError::Sqlx(err)
    }
}

impl From<ValidError> for MfaError {
    fn from(err: ValidError) -> Self {
        MfaError::ValidParam(err)
    }
}

pub type MfaResult<T> = Result<T, MfaError>;
