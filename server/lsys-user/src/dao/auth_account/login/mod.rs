use crate::dao::AccountDao;
use crate::dao::AccountError;
use crate::dao::AccountResult;
use crate::dao::UserAuthError;
use crate::dao::UserAuthResult;
use crate::model::AccountModel;
use async_trait::async_trait;
use futures_util::Future;
use lsys_access::dao::SessionBody;
use lsys_core::fluent_message;
use lsys_core::IntoFluentMessage;
use std::net::IpAddr;
use std::sync::Arc;

pub struct AccountLoginEnv {
    pub login_ip: Option<IpAddr>,
}

pub trait AccountLoginMeta {
    fn login_type() -> String;
    fn login_timeout() -> u64 {
        3 * 24 * 3600
    }
}

#[async_trait]
pub trait AccountLoginParam {
    type Meta: AccountLoginMeta;
    fn account_name(&self) -> String;
    //获取登录用户数据
    async fn get_account(
        &self,
        login_env: &AccountLoginEnv,
    ) -> AccountResult<(String, AccountModel)>;
}

async fn reload_match_wrap<D: AccountLoginMeta>(
    session: &SessionBody,
    res: impl Future<Output = AccountResult<String>>,
) -> Option<UserAuthResult<(String, u64)>> {
    if session.session().login_type == D::login_type() {
        Some(
            res.await
                .map_err(|e| match e {
                    AccountError::Sqlx(e) => UserAuthError::Sqlx(e),
                    AccountError::System(e) => UserAuthError::System(e),
                    AccountError::Status(e) => UserAuthError::System(e.1),
                    AccountError::Redis(e) => UserAuthError::Redis(e),
                    AccountError::RedisPool(e) => UserAuthError::RedisPool(e),
                    AccountError::ValidCode(e) => UserAuthError::System(e.to_fluent_message()),
                    AccountError::Setting(e) => UserAuthError::System(e.to_fluent_message()),
                    AccountError::Param(e) => UserAuthError::System(e.to_fluent_message()),
                    AccountError::AuthStatusError(e) => {
                        UserAuthError::System(e.1.to_fluent_message())
                    }
                    AccountError::UserAuthError(e) => UserAuthError::System(e.to_fluent_message()),
                    AccountError::AccessError(e) => UserAuthError::AccessError(e),
                    AccountError::PasswordNotMatch(e) => {
                        UserAuthError::System(e.1.to_fluent_message())
                    }
                    AccountError::PasswordNotSet(e) => {
                        UserAuthError::System(e.1.to_fluent_message())
                    }
                    AccountError::UserNotFind(e) => UserAuthError::System(e.to_fluent_message()),
                    AccountError::SerdeJson(e) => {
                        UserAuthError::System(fluent_message!("serde-json-error", e))
                    }
                })
                .map(|e| (e, D::login_timeout())),
        )
    } else {
        None
    }
}

/// 检测指定用户的密码是否正确并返回验证结果
async fn auth_check_account_password(
    account: &Arc<AccountDao>,
    user: AccountModel,
    check_password: &str,
) -> AccountResult<AccountModel> {
    if user.password_id > 0 {
        if !account
            .account_password
            .check_password(&user, check_password)
            .await?
        {
            return AccountResult::Err(AccountError::PasswordNotMatch((
                user.id,
                fluent_message!("auth-bad-password"), //"User bad password"
            )));
        }
        AccountResult::Ok(user)
    } else {
        AccountResult::Err(AccountError::PasswordNotSet((
            user.id,
            fluent_message!("auth-not-set-password"), // "User not set password"
        )))
    }
}

#[macro_use]
mod macros;
mod account_email;
mod account_email_code;
mod account_external;
mod account_mobile;
mod account_mobile_code;
mod account_name;
pub use self::account_email::*;
pub use self::account_email_code::*;
pub use self::account_external::*;
pub use self::account_mobile::*;
pub use self::account_mobile_code::*;
pub use self::account_name::*;
