mod account;
mod auth;
mod oauth;

pub use account::*;
pub use auth::*;
use lsys_logger::dao::ChangeLoggerDao;
use lsys_user::dao::UserDao;
pub use oauth::*;

use super::{AppArea, AppCaptcha, AppSender};
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub struct WebUser {
    pub account: Arc<WebUserAccount>,            //account一些扩充
    pub auth: Arc<WebUserAuth>,                  //auth一些扩充
    pub user_dao: Arc<UserDao>,                  //内置用户模块 UserDao 模块
    pub change_logger_dao: Arc<ChangeLoggerDao>, //操作日志模块 ChangeLoggerDao 模块
}

impl WebUser {
    pub fn new(
        db: Pool<MySql>,
        user_dao: Arc<UserDao>,
        sender: Arc<AppSender>,
        captcha: Arc<AppCaptcha>,
        area: Arc<AppArea>,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        WebUser {
            account: Arc::new(WebUserAccount::new(
                user_dao.clone(),
                captcha.clone(),
                sender.clone(),
                area.clone(),
                db.clone(),
            )),
            auth: Arc::new(WebUserAuth::new(
                db.clone(),
                user_dao.clone(),
                captcha,
                sender,
            )),
            user_dao,
            change_logger_dao: logger,
        }
    }
}
