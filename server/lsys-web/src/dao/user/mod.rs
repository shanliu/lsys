use lsys_core::AppCore;
use lsys_rbac::dao::RbacDao;
use lsys_setting::dao::Setting;
use lsys_user::dao::{
    auth::{LoginData, SessionData, SessionUserData, UserAuthData, UserAuthRedisStore},
    UserDao,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{MySql, Pool};
use std::sync::Arc;

use super::captcha::WebAppCaptcha;

mod del;
mod detail;
mod list_user;
mod login;
mod oauth;
mod register;
pub use detail::UserDataOption;

pub use register::UserRegData;

pub struct WebUser {
    pub user_dao: Arc<UserDao<UserAuthRedisStore>>,
    pub rbac_dao: Arc<RbacDao>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
    pub captcha: Arc<WebAppCaptcha>,
    pub app_core: Arc<AppCore>,
    pub setting: Arc<Setting>,
}

impl WebUser {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        user_dao: Arc<UserDao<UserAuthRedisStore>>,
        rbac_dao: Arc<RbacDao>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        captcha: Arc<WebAppCaptcha>,
        app_core: Arc<AppCore>,

        setting: Arc<Setting>,
    ) -> Self {
        WebUser {
            user_dao,
            rbac_dao,
            captcha,
            db,
            redis,
            app_core,
            setting,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowUserAuthData {
    pub login_type: &'static str,
    pub login_data: Value,
    pub user_id: u64,
    pub user_nickname: String,
    pub user_password_id: u64,
    pub time_out: u64,
    pub login_time: u64,
}

impl From<UserAuthData> for ShowUserAuthData {
    fn from(auth_data: UserAuthData) -> Self {
        let SessionUserData {
            user_id,
            user_nickname,
            user_password_id,
            time_out,
        } = auth_data.user_data().to_owned();
        let UserAuthData {
            login_type,
            login_data,
            ..
        } = auth_data;
        let (show_login_type, show_login_data) = match login_data {
            LoginData::Name(val) => ("name", json!(val)),
            LoginData::Email(val) => ("email", json!(val)),
            LoginData::EmailCode(val) => ("email-code", json!(val)),
            LoginData::Mobile(val) => ("sms", json!(val)),
            LoginData::MobileCode(val) => ("sms-code", json!(val)),
            LoginData::External(val) => ("external", json!(val)),
        };
        let stime = login_type.time_out as u64;
        let login_time = if time_out > stime {
            time_out - stime
        } else {
            0
        };
        ShowUserAuthData {
            login_type: show_login_type,
            login_data: show_login_data,
            user_id,
            user_nickname,
            user_password_id,
            time_out,
            login_time,
        }
    }
}
