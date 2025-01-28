//auth一些扩充
mod login;
mod login_data;
mod oauth;
mod password;
mod register;

use std::sync::Arc;

pub use login::*;
pub use login_data::*;
use lsys_user::dao::UserDao;
pub use password::*;
pub use register::*;
use sqlx::{MySql, Pool};

use crate::dao::{AppCaptcha, AppSender};

pub struct WebUserAuth {
    user_dao: Arc<UserDao>,
    captcha: Arc<AppCaptcha>,
    sender: Arc<AppSender>,
    db: Pool<sqlx::MySql>,
}

impl WebUserAuth {
    pub fn new(
        db: Pool<MySql>,
        user_dao: Arc<UserDao>,
        captcha: Arc<AppCaptcha>,
        sender: Arc<AppSender>,
    ) -> Self {
        WebUserAuth {
            user_dao,
            captcha,
            sender,
            db,
        }
    }
}
