//account 的一些扩充
mod address;
mod del;
mod detail;
mod email;
mod info;
mod list;
mod mobile;

pub use address::*;
pub use detail::*;
pub use info::*;
use lsys_access::dao::UserInfo;
use lsys_user::dao::UserDao;
use sqlx::Pool;
use std::sync::Arc;

use crate::{
    common::JsonResult,
    dao::{AppArea, AppCaptcha, AppSender},
};

pub struct WebUserAccount {
    user_dao: Arc<UserDao>,
    captcha: Arc<AppCaptcha>,
    sender: Arc<AppSender>,
    area: Arc<AppArea>,
    db: Pool<sqlx::MySql>,
}

impl WebUserAccount {
    pub fn new(
        user_dao: Arc<UserDao>,
        captcha: Arc<AppCaptcha>,
        sender: Arc<AppSender>,
        area: Arc<AppArea>,
        db: Pool<sqlx::MySql>,
    ) -> Self {
        WebUserAccount {
            user_dao,
            captcha,
            sender,
            area,
            db,
        }
    }
    //转换 account_id 为用户数据
    pub async fn account_id_to_user(&self, account_id: u64) -> JsonResult<UserInfo> {
        let account = self
            .user_dao
            .account_dao
            .account
            .cache()
            .find_by_id(&account_id)
            .await?;
        Ok(self
            .user_dao
            .account_dao
            .account
            .cache()
            .get_user(&account)
            .await?)
    }
}
