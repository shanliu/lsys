use crate::common::JsonData;
use lsys_access::dao::AccessError;
use lsys_app::dao::AppError;
use lsys_app_sender::dao::SenderError;
use lsys_core::app_core::AppCoreError;
use lsys_core::fluent_message;
use lsys_core::fluents::{FluentMessage, IntoFluentMessage};
use lsys_core::remote_notify::RemoteNotifyError;
use lsys_core::valid_code::ValidCodeError;
use lsys_core::valid_param::ValidError;
use lsys_files::common::FileError;
use lsys_lib_area::AreaError;
use lsys_mfa::dao::MfaError;
use lsys_rbac::dao::RbacError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{AccountError, UserAuthError};

pub enum WebError {
    AreaError(AreaError),
    AppCore(AppCoreError),
    RemoteNotifyError(RemoteNotifyError),
    Sqlx(sqlx::Error),
    SettingError(SettingError),
    SenderError(SenderError),
    RbacError(RbacError),
    AccountError(AccountError),
    AccessError(AccessError),
    UserAuthError(UserAuthError),
    AppError(AppError),
    MfaError(MfaError),
    FileError(FileError),
    ValidError(ValidError),
    ValidCodeError(ValidCodeError),
    Message(FluentMessage),
    JsonResponse(Box<JsonData>, FluentMessage),
}

impl IntoFluentMessage for WebError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            WebError::AreaError(err) => fluent_message!("lsys-lib-area-error", err),
            WebError::AppCore(err) => err.to_fluent_message(),
            WebError::RemoteNotifyError(err) => err.to_fluent_message(),
            WebError::Sqlx(err) => fluent_message!("sqlx-error", err),
            WebError::SettingError(err) => err.to_fluent_message(),
            WebError::SenderError(err) => err.to_fluent_message(),
            WebError::RbacError(err) => err.to_fluent_message(),
            WebError::AccountError(err) => err.to_fluent_message(),
            WebError::UserAuthError(err) => err.to_fluent_message(),
            WebError::AppError(err) => err.to_fluent_message(),
            WebError::MfaError(err) => err.to_fluent_message(),
            WebError::FileError(err) => err.to_fluent_message(),
            WebError::ValidError(err) => err.to_fluent_message(),
            WebError::ValidCodeError(err) => err.to_fluent_message(),
            WebError::Message(err) => err.to_owned(),
            WebError::JsonResponse(_, err) => err.to_owned(),
            WebError::AccessError(err) => err.to_fluent_message(),
        }
    }
}

impl From<lsys_lib_area::AreaError> for WebError {
    fn from(value: lsys_lib_area::AreaError) -> Self {
        Self::AreaError(value)
    }
}

impl From<AppCoreError> for WebError {
    fn from(value: AppCoreError) -> Self {
        Self::AppCore(value)
    }
}

impl From<RemoteNotifyError> for WebError {
    fn from(value: RemoteNotifyError) -> Self {
        Self::RemoteNotifyError(value)
    }
}

impl From<sqlx::Error> for WebError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sqlx(value)
    }
}

impl From<SettingError> for WebError {
    fn from(value: SettingError) -> Self {
        Self::SettingError(value)
    }
}

impl From<SenderError> for WebError {
    fn from(value: SenderError) -> Self {
        Self::SenderError(value)
    }
}

impl From<RbacError> for WebError {
    fn from(value: RbacError) -> Self {
        Self::RbacError(value)
    }
}

impl From<AccountError> for WebError {
    fn from(value: AccountError) -> Self {
        Self::AccountError(value)
    }
}
impl From<AccessError> for WebError {
    fn from(value: AccessError) -> Self {
        Self::AccessError(value)
    }
}

impl From<UserAuthError> for WebError {
    fn from(value: UserAuthError) -> Self {
        Self::UserAuthError(value)
    }
}

impl From<AppError> for WebError {
    fn from(value: AppError) -> Self {
        Self::AppError(value)
    }
}

impl From<MfaError> for WebError {
    fn from(value: MfaError) -> Self {
        Self::MfaError(value)
    }
}

impl From<FileError> for WebError {
    fn from(value: FileError) -> Self {
        Self::FileError(value)
    }
}

impl From<ValidError> for WebError {
    fn from(value: ValidError) -> Self {
        Self::ValidError(value)
    }
}

impl From<ValidCodeError> for WebError {
    fn from(value: ValidCodeError) -> Self {
        Self::ValidCodeError(value)
    }
}

impl From<std::io::Error> for WebError {
    fn from(value: std::io::Error) -> Self {
        Self::Message(lsys_core::fluent_message!("io-error", value))
    }
}

pub type WebResult<T> = Result<T, WebError>;
