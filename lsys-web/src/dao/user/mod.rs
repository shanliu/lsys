use lsys_core::{AppCore, FluentMessage};
use lsys_rbac::dao::RbacDao;
use lsys_user::dao::{
    auth::{LoginData, SessionData, SessionUserData, UserAuthData, UserAuthRedisStore},
    UserDao,
};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{MySql, Pool};
use std::sync::Arc;

use crate::RelationParam;

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
    fluent: Arc<FluentMessage>,
}

impl WebUser {
    pub fn new(
        user_dao: Arc<UserDao<UserAuthRedisStore>>,
        rbac_dao: Arc<RbacDao>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        captcha: Arc<WebAppCaptcha>,
        app_core: Arc<AppCore>,
        fluent: Arc<FluentMessage>,
    ) -> Self {
        WebUser {
            user_dao,
            rbac_dao,
            captcha,
            db,
            redis,
            app_core,
            fluent,
        }
    }
    pub async fn user_relation_key(
        &self,
        user_auth_data: &SessionUserData,
        res_user_id: &[u64],
    ) -> Vec<RelationParam> {
        //todo 待定
        let mut rk = vec![];
        if res_user_id.contains(&10) {
            rk.push(RelationParam {
                role_key: "vip1".to_string(),
                user_id: 0,
            }); //一些等级角色
        }
        if user_auth_data.user_id == 1 && res_user_id.contains(&2) {
            rk.push(RelationParam {
                role_key: "firend".to_string(),
                user_id: 2,
            }); //用户资源
        }
        rk
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
