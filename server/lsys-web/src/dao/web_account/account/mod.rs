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
use lsys_access::dao::{AccessDao, UserInfo};
use lsys_user::dao::UserDao;
use sqlx::Pool;
use std::sync::Arc;

use crate::dao::{AppArea, AppCaptcha, AppSender, WebResult};

pub struct WebUserAccount {
    user_dao: Arc<UserDao>,
    access_dao: Arc<AccessDao>,
    captcha: Arc<AppCaptcha>,
    sender: Arc<AppSender>,
    area: Arc<AppArea>,
    db: Pool<sqlx::MySql>,
}

impl WebUserAccount {
    pub fn new(
        user_dao: Arc<UserDao>,
        access_dao: Arc<AccessDao>,
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
            access_dao,
        }
    }
    //转换 account_id 为用户数据
    pub async fn account_id_to_user(&self, account_id: u64) -> WebResult<UserInfo> {
        let account = self
            .user_dao
            .account_dao
            .account
            .cache()
            .find_by_id(&account_id)
            .await?;
        Ok(self
            .access_dao
            .user
            .cache()
            .sync_user(0, account.id, Some(&account.nickname), None)
            .await?)
    }
}
