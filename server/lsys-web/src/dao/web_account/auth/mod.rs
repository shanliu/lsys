//auth一些扩充
mod login;
mod login_data;
mod oauth;
mod password;
mod register;

use std::sync::Arc;

pub use login::*;
pub use login_data::*;
use lsys_access::dao::AccessDao;
use lsys_app::dao::AppDao;
use lsys_user::dao::UserDao;
pub use password::*;
pub use register::*;
use sqlx::{MySql, Pool};

use crate::dao::{AppCaptcha, AppSender, WebMfa};

pub struct WebUserAuth {
    user_dao: Arc<UserDao>,
    app_dao: Arc<AppDao>,
    captcha: Arc<AppCaptcha>,
    sender: Arc<AppSender>,
    db: Pool<sqlx::MySql>,
    mfa: Arc<WebMfa>,
    access_dao: Arc<AccessDao>,
}

impl WebUserAuth {
    pub fn new(
        db: Pool<MySql>,
        user_dao: Arc<UserDao>,
        app_dao: Arc<AppDao>,
        access_dao: Arc<AccessDao>,
        captcha: Arc<AppCaptcha>,
        sender: Arc<AppSender>,
        mfa: Arc<WebMfa>,
    ) -> Self {
        WebUserAuth {
            user_dao,
            app_dao,
            captcha,
            sender,
            db,
            mfa,
            access_dao,
        }
    }
}
