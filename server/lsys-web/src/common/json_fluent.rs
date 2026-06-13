// 定制错误的输出附带信息

use lsys_access::dao::AccessError;
use lsys_app_sender::dao::SenderError;
use lsys_core::{
    app_core::AppCoreError, config::ConfigError, fluents::FluentBundle, remote_notify::RemoteNotifyError, secret::SecretError, valid_code::ValidCodeError, valid_param::ValidError
};

use lsys_logger::dao::LoggerError;
use lsys_mfa::dao::MfaError;
use lsys_rbac::dao::RbacError;
use lsys_setting::dao::SettingError;
use lsys_user::dao::{AccountError, UserAuthError};
use serde_json::json;

use std::{collections::HashMap, num::ParseIntError};

use lsys_app::dao::AppError;

use crate::dao::WebError;

use super::{JsonData, JsonFluent};

//lib error

impl JsonFluent for sqlx::Error {
    fn to_json_data(&self, _: &FluentBundle) -> JsonData {
        match self {
            sqlx::Error::RowNotFound => JsonData::default().set_sub_code("not_found").set_code(404),
            _ => JsonData::default().set_code(500).set_sub_code("sqlx"),
        }
    }
}
macro_rules! crate_error_fluent {
    ($crate_error:ty,$code:literal) => {
        impl JsonFluent for $crate_error {
            fn to_json_data(&self, _: &FluentBundle) -> JsonData {
                JsonData::default().set_code(500).set_sub_code($code)
            }
        }
    };
}
crate_error_fluent!(config::ConfigError, "config");
crate_error_fluent!(std::io::Error, "io");
crate_error_fluent!(tera::Error, "tera");
crate_error_fluent!(redis::RedisError, "redis");
crate_error_fluent!(deadpool_redis::PoolError, "redis");
crate_error_fluent!(serde_json::Error, "serde");
crate_error_fluent!(ParseIntError, "parse");
crate_error_fluent!(std::string::FromUtf8Error, "parse");

//内部错误

impl JsonFluent for AccountError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default()
            .set_code(500)
            .set_sub_code("user_account");
        match self {
            AccountError::Param(_) => json_data,
            AccountError::System(_) => json_data,
            AccountError::Status(_) => json_data,
            AccountError::Redis(err) => err.to_json_data(fluent),
            AccountError::RedisPool(err) => err.to_json_data(fluent),
            AccountError::ValidCode(err) => err.to_json_data(fluent),
            AccountError::Sqlx(err) => err.to_json_data(fluent),
            AccountError::Setting(err) => err.to_json_data(fluent),
            AccountError::SerdeJson(err) => err.to_json_data(fluent),

            AccountError::UserAuthError(err) => err.to_json_data(fluent),
            AccountError::AccessError(err) => err.to_json_data(fluent),

            AccountError::PasswordNotMatch((_, _)) => json_data.set_sub_code("password_wrong"),
            AccountError::PasswordNotSet(_) => json_data.set_sub_code("password_empty"),
            AccountError::AuthStatusError(_) => json_data.set_sub_code("status_wrong"),
            AccountError::UserNotFind(_) => json_data.set_sub_code("not_find"),
            AccountError::Vaild(err) => err.to_json_data(fluent),
            AccountError::MfaNeed { mfa_token, .. } => {
                json_data.set_sub_code("mfa_need").set_body(json!({
                    "mfa_token":mfa_token
                }))
            }
            AccountError::MfaError(err) => err.to_json_data(fluent),
        }
    }
}

impl JsonFluent for ValidCodeError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500);
        match self {
            ValidCodeError::NotMatch(err) => json_data.set_sub_code("valid_code").set_body(json!({
                "type":err.prefix
            })),
            ValidCodeError::Utf8Err(_) => json_data.set_sub_code("valid_code_err"),
            ValidCodeError::Tag(_) => json_data.set_sub_code("valid_code_err"),
            ValidCodeError::Redis(err) => err.to_json_data(fluent),
            ValidCodeError::RedisPool(err) => err.to_json_data(fluent),
            ValidCodeError::Valid(err) => err.to_json_data(fluent),
            ValidCodeError::Serialize(_) => json_data.set_sub_code("serialize_error"),
        }
    }
}

impl JsonFluent for ValidError {
    fn to_json_data(&self, _: &FluentBundle) -> JsonData {
        JsonData::default()
            .set_code(400)
            .set_body(self.to_value())
            .set_sub_code("bad_param")
    }
}
impl JsonFluent for UserAuthError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500);
        match self {
            UserAuthError::NotLogin(_) => json_data.set_code(403).set_sub_code("not_login"),
            UserAuthError::System(_) => json_data.set_sub_code("auth"),
            UserAuthError::CheckCaptchaNeed(_) => json_data.set_sub_code("need_captcha"),
            UserAuthError::CheckUserLock((time, _)) => {
                json_data.set_sub_code("user_lock").set_body(json!({
                    "lock_time":time
                }))
            }
            UserAuthError::TokenParse(_) => json_data.set_sub_code("token_wrong"),
            UserAuthError::Sqlx(err) => err.to_json_data(fluent),
            UserAuthError::ValidCode(err) => err.to_json_data(fluent),
            UserAuthError::Redis(err) => err.to_json_data(fluent),
            UserAuthError::RedisPool(err) => err.to_json_data(fluent),
            UserAuthError::Utf8Err(err) => err.to_json_data(fluent),
            UserAuthError::AccessError(err) => err.to_json_data(fluent),
            UserAuthError::Vaild(err) => err.to_json_data(fluent),
        }
    }
}

impl JsonFluent for AccessError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_sub_code("access").set_code(500);
        match self {
            AccessError::LoginTokenDataExit(_) => {
                json_data.set_code(400).set_sub_code("token_data")
            }
            AccessError::Sqlx(err) => err.to_json_data(fluent),
            AccessError::NotLogin => json_data.set_code(403).set_sub_code("not_login"),
            AccessError::IsLogout => json_data.set_code(403).set_sub_code("not_login"),
            AccessError::System(_) => json_data,
            AccessError::SerdeJson(err) => err.to_json_data(fluent),
            AccessError::BadAccount(_) => json_data.set_code(400).set_sub_code("bad_account"),
            AccessError::Vaild(err) => err.to_json_data(fluent),
        }
    }
}
impl JsonFluent for RbacError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_sub_code("rbac").set_code(500);
        match self {
            RbacError::Sqlx(err) => err.to_json_data(fluent),
            RbacError::Check(err) => {
                let mut hash = HashMap::<String, Vec<String>>::new();
                for tmp in err {
                    let key = if tmp.res_data.is_empty() {
                        tmp.res_type.to_string()
                    } else {
                        format!("{}-{}", tmp.res_type, tmp.res_data)
                    };
                    hash.entry(key)
                        .or_default()
                        .push(fluent.format_message(&tmp.msg));
                }
                json_data
                    .set_code(403)
                    .set_sub_code("check_fail")
                    .set_body(json!( {
                        "check_detail":hash,
                    }))
            }
            RbacError::System(_) => json_data,
            RbacError::Vaild(err) => err.to_json_data(fluent),
        }
    }
}

impl JsonFluent for SettingError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            SettingError::Sqlx(err) => err.to_json_data(fluent),
            SettingError::SerdeJson(err) => err.to_json_data(fluent),
            SettingError::Vaild(err) => err.to_json_data(fluent),
        }
    }
}

impl JsonFluent for SenderError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500).set_sub_code("sender");
        match self {
            SenderError::Sqlx(err) => err.to_json_data(fluent),
            SenderError::Redis(err) => err.to_json_data(fluent),
            SenderError::RedisPool(err) => err.to_json_data(fluent),
            SenderError::Tera(err) => err.to_json_data(fluent),
            SenderError::Setting(err) => err.to_json_data(fluent),
            SenderError::System(_) => json_data,
            SenderError::Vaild(err) => err.to_json_data(fluent),
            SenderError::AppCore(error) => error.to_json_data(fluent),
        }
    }
}

impl JsonFluent for AppError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        let json_data = JsonData::default().set_code(500).set_sub_code("apps");
        match self {
            AppError::ScopeBad(_) => json_data.set_sub_code("app-bad-scope"),
            AppError::System(_) => json_data,
            AppError::Sqlx(err) => err.to_json_data(fluent),
            AppError::Redis(err) => err.to_json_data(fluent),
            AppError::RedisPool(err) => err.to_json_data(fluent),
            AppError::AppCore(err) => err.to_json_data(fluent),
            AppError::SerdeJson(err) => err.to_json_data(fluent),
            AppError::Access(err) => err.to_json_data(fluent),
            AppError::AppNotFound(_) => json_data.set_sub_code("app-not-found"),
            AppError::AppBadStatus => json_data.set_sub_code("app-bad-status"),
            AppError::AppBadFeature(_, _) => json_data.set_sub_code("app-bad-feature"),
            AppError::AppOAuthClientBadConfig(_) => json_data,
            AppError::AppOAuthClientBadDomain(_) => json_data,
            AppError::Vaild(err) => err.to_json_data(fluent),
        }
    }
}

impl JsonFluent for ConfigError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            ConfigError::Io(err) => err.to_json_data(fluent),
            ConfigError::Config(err) => err.to_json_data(fluent),
        }
        .set_code(500)
        .set_sub_code("config")
    }
}
impl JsonFluent for RemoteNotifyError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            RemoteNotifyError::System(_) => JsonData::error(),
            RemoteNotifyError::RedisPool(pool_error) => pool_error.to_json_data(fluent),
            RemoteNotifyError::Redis(redis_error) => redis_error.to_json_data(fluent),
            RemoteNotifyError::RemoteTimeOut => JsonData::error(),
        }
    }
}

impl JsonFluent for LoggerError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            LoggerError::Sqlx(err) => err.to_json_data(fluent),
        }
    }
}
impl JsonFluent for SecretError {
    fn to_json_data(&self, _: &FluentBundle) -> JsonData {
        match self {
            SecretError::KeyNotFound(key) => JsonData::default()
                .set_code(400)
                .set_sub_code("secret_key_not_found")
                .set_body(json!({ "key": key })),
            SecretError::Config(detail) => JsonData::default()
                .set_code(500)
                .set_sub_code("secret_config_error")
                .set_body(json!({ "detail": detail })),
            SecretError::KmsNotFound(name) => JsonData::default()
                .set_code(500)
                .set_sub_code("secret_kms_not_found")
                .set_body(json!({ "kms_name": name })),
            SecretError::KmsDecrypt(detail) => JsonData::default()
                .set_code(500)
                .set_sub_code("secret_kms_error")
                .set_body(json!({ "detail": detail })),
            SecretError::Decode(detail) => JsonData::default()
                .set_code(500)
                .set_sub_code("secret_decode_error")
                .set_body(json!({ "detail": detail })),
            SecretError::Encrypt(detail) => JsonData::default()
                .set_code(500)
                .set_sub_code("secret_encrypt_error")
                .set_body(json!({ "detail": detail })),
        }
    }
}
impl JsonFluent for WebError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            WebError::AppCore(err) => err.to_json_data(fluent),
            WebError::AreaError(err) => err.to_json_data(fluent),
            WebError::Message(_) => JsonData::error(),
            WebError::JsonResponse(e, _) => *e.to_owned(),
            WebError::RemoteNotifyError(err) => err.to_json_data(fluent),
            WebError::Sqlx(err) => err.to_json_data(fluent),
            WebError::SettingError(err) => err.to_json_data(fluent),
            WebError::SenderError(err) => err.to_json_data(fluent),
            WebError::RbacError(err) => err.to_json_data(fluent),
            WebError::AccountError(err) => err.to_json_data(fluent),
            WebError::UserAuthError(err) => err.to_json_data(fluent),
            WebError::AppError(err) => err.to_json_data(fluent),
            WebError::MfaError(err) => err.to_json_data(fluent),
            WebError::ValidError(err) => err.to_json_data(fluent),
            WebError::ValidCodeError(err) => err.to_json_data(fluent),
            WebError::AccessError(err) => err.to_json_data(fluent),
            WebError::FileError(err) => err.to_json_data(fluent),
            WebError::FileManagerError(err) => err.to_json_data(fluent),
            WebError::LoggerError(err) => err.to_json_data(fluent),
            WebError::SecretError(err) =>err.to_json_data(fluent),
        }
    }
}

impl JsonFluent for MfaError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            MfaError::Sqlx(err) => err.to_json_data(fluent),
            MfaError::ValidParam(err) => err.to_json_data(fluent),
            MfaError::NotEnabled => JsonData::default()
                .set_code(500)
                .set_sub_code("mfa-not-enabled"),
            MfaError::VerifyFailed => JsonData::default()
                .set_code(500)
                .set_sub_code("mfa-verify-failed"),
            MfaError::Replay => JsonData::default()
                .set_code(500)
                .set_sub_code("mfa-verify-failed"),
            MfaError::SecretInvalid => JsonData::default()
                .set_code(500)
                .set_sub_code("mfa-secret-invalid"),
            MfaError::TokenExpired => JsonData::default()
                .set_code(500)
                .set_sub_code("mfa-token-expired"),
        }
    }
}

impl JsonFluent for AppCoreError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            AppCoreError::Sqlx(error) => error.to_json_data(fluent),
            AppCoreError::Env(_) => JsonData::default().set_code(500).set_sub_code("var"),
            AppCoreError::Tera(error) => error.to_json_data(fluent),
            AppCoreError::Io(error) => error.to_json_data(fluent),
            AppCoreError::System(_) => JsonData::default().set_code(500).set_sub_code("system"),
            AppCoreError::Log(_) => JsonData::default().set_code(500).set_sub_code("system"),
            AppCoreError::Redis(error) => error.to_json_data(fluent),
            AppCoreError::RedisCreatePool(_) => {
                JsonData::default().set_code(500).set_sub_code("redis")
            }
            AppCoreError::RedisPool(error) => error.to_json_data(fluent),
            AppCoreError::Dotenv(_) => JsonData::default().set_code(500).set_sub_code("env"),
            AppCoreError::AppDir(_) => JsonData::default().set_code(500).set_sub_code("system"),
            AppCoreError::Config(error) => error.to_json_data(fluent),
            AppCoreError::Fluent(_) => JsonData::default().set_code(500).set_sub_code("fluent"),
            AppCoreError::RemoteNotify(_) => {
                JsonData::default().set_code(500).set_sub_code("notify")
            }
        }
    }
}

impl JsonFluent for lsys_lib_area::AreaError {
    fn to_json_data(&self, _: &FluentBundle) -> JsonData {
        match self {
            lsys_lib_area::AreaError::DB(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("lsys_lib_area"),
            lsys_lib_area::AreaError::System(_) => {
                JsonData::default().set_code(500).set_sub_code("area")
            }
            lsys_lib_area::AreaError::NotFind(_) => JsonData::default().set_sub_code("not_found"),
            lsys_lib_area::AreaError::Store(_) => JsonData::default().set_sub_code("area_store"),
            lsys_lib_area::AreaError::Tantivy(_) => {
                JsonData::default().set_sub_code("area_tantivy")
            }
        }
    }
}

impl JsonFluent for lsys_file::common::FileError {
    fn to_json_data(&self, fluent: &FluentBundle) -> JsonData {
        match self {
            lsys_file::common::FileError::Sqlx(e) => e.to_json_data(fluent),
            lsys_file::common::FileError::Io(e) => e.to_json_data(fluent),
            lsys_file::common::FileError::Redis(_) => {
                JsonData::default().set_code(500).set_sub_code("file-redis")
            }
            lsys_file::common::FileError::RedisPool(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("file-redis-pool"),
            lsys_file::common::FileError::AppCore(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("file-app-core"),
            lsys_file::common::FileError::Setting(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("file-setting"),
            lsys_file::common::FileError::Valid(_) => {
                JsonData::default().set_code(400).set_sub_code("file-valid")
            }
            lsys_file::common::FileError::System(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("file-system"),
            lsys_file::common::FileError::Param(_) => {
                JsonData::default().set_code(400).set_sub_code("file-param")
            }
            lsys_file::common::FileError::Http(_) => {
                JsonData::default().set_code(500).set_sub_code("file-http")
            }
            lsys_file::common::FileError::InvalidStatusCode(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("file-status-code"),
            lsys_file::common::FileError::RedirectLimitExceeded => JsonData::default()
                .set_code(500)
                .set_sub_code("file-redirect"),
            lsys_file::common::FileError::InvalidChunkData(_) => {
                JsonData::default().set_code(400).set_sub_code("file-chunk")
            }
            lsys_file::common::FileError::DownloadTimeout(_, _) => JsonData::default()
                .set_code(504)
                .set_sub_code("file-download-timeout"),
            lsys_file::common::FileError::DownloadFailed(_, _) => JsonData::default()
                .set_code(500)
                .set_sub_code("file-download-failed"),
            lsys_file::common::FileError::Lock(
                lsys_core::dist_lock::DistLockError::AcquireFailed { .. },
            ) => JsonData::default()
                .set_code(409)
                .set_sub_code("file-merge-retry"),
            lsys_file::common::FileError::Lock(_) => JsonData::default()
                .set_code(500)
                .set_sub_code("file-lock-failed"),
            lsys_file::common::FileError::InvalidFileKey(_) => JsonData::default()
                .set_code(400)
                .set_sub_code("file-invalid-key"),
            lsys_file::common::FileError::FileKeyExpired(_) => JsonData::default()
                .set_code(400)
                .set_sub_code("file-key-expired"),
        }
    }
}

impl JsonFluent for lsys_file_manager::dao::FileManagerError {
    fn to_json_data(&self, _: &FluentBundle) -> JsonData {
        JsonData::default()
            .set_code(500)
            .set_sub_code("file-manager")
    }
}
