//设置模块扩充

mod site_config;
pub use site_config::*;

use std::sync::Arc;

use lsys_setting::dao::SettingDao;

pub struct WebSetting {
    pub setting_dao: Arc<SettingDao>,
    db: sqlx::Pool<sqlx::MySql>,
}

impl WebSetting {
    pub fn new(setting_dao: Arc<SettingDao>, db: sqlx::Pool<sqlx::MySql>) -> Self {
        Self { setting_dao, db }
    }
}
