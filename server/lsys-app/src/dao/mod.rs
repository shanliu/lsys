use lsys_core::{AppCoreError, RemoteNotify};
use lsys_user::dao::account::UserAccount;

use self::app::{Apps, AppsOauth, SubApps};
use lsys_logger::dao::ChangeLogger;
use sqlx::{MySql, Pool};
use std::sync::Arc;
pub mod app;

mod result;
pub mod session;
pub use result::*;

pub struct AppDao {
    //内部依赖
    pub app: Arc<Apps>,
    pub sub_app: Arc<SubApps>,
    pub app_oauth: Arc<AppsOauth>,
    pub db: Pool<MySql>,
    pub redis: deadpool_redis::Pool,
}

impl AppDao {
    pub async fn new(
        user_account: Arc<UserAccount>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLogger>,
        time_out: u64,
    ) -> Result<AppDao, AppCoreError> {
        let sub_app = Arc::from(SubApps::new(
            db.clone(),
            remote_notify.clone(),
            logger.clone(),
        ));
        let app = Arc::from(Apps::new(
            db.clone(),
            remote_notify.clone(),
            // fluent.clone(),
            logger,
            sub_app.clone(),
        ));
        let app_oauth = Arc::from(AppsOauth::new(
            app.clone(),
            user_account,
            db.clone(),
            redis.clone(),
            // fluent.clone(),
            remote_notify,
            time_out,
        ));
        Ok(AppDao {
            app,
            sub_app,
            app_oauth,
            db,
            // fluent,
            redis,
        })
    }
}
