#[macro_use]
mod macros;
use crate::dao::account::UserAccount;
use crate::dao::auth::UserAuthResult;

use crate::model::UserModel;
use lsys_core::{get_message, FluentMessage};
use std::sync::Arc;

use super::UserAuthError;

/// 检测指定用户的密码是否正确并返回验证结果
pub(crate) async fn auth_check_user_password(
    fluent: &Arc<FluentMessage>,
    account: &Arc<UserAccount>,
    user: UserModel,
    check_password: &String,
) -> UserAuthResult<UserModel> {
    if user.password_id > 0 {
        if !account
            .user_password
            .check_password(&user, check_password)
            .await?
        {
            return UserAuthResult::Err(UserAuthError::PasswordNotMatch((
                user.id,
                get_message!(fluent, "auth-bad-password", "User bad password"),
            )));
        }
        UserAuthResult::Ok(user)
    } else {
        UserAuthResult::Err(UserAuthError::PasswordNotSet((
            user.id,
            get_message!(fluent, "auth-not-set-password", "User not set password"),
        )))
    }
}

mod param_email;
mod param_email_code;
mod param_external;
mod param_mobile;
mod param_mobile_code;
mod param_name;
pub use self::param_email::*;
pub use self::param_email_code::*;
pub use self::param_external::*;
pub use self::param_mobile::*;
pub use self::param_mobile_code::*;
pub use self::param_name::*;
