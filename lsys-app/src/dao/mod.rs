use lsys_core::{AppCore, AppCoreError, FluentMessage};
use lsys_user::dao::account::UserAccount;
use redis::aio::ConnectionManager;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::sync::Mutex;

use self::app::{Apps, AppsOauth};

pub mod app;
pub mod session;

pub struct AppDao {
    //内部依赖
    pub app: Arc<Apps>,
    pub app_oauth: Arc<AppsOauth>,
    pub db: Pool<MySql>,
    pub redis: Arc<Mutex<ConnectionManager>>,
    pub(crate) fluent: Arc<FluentMessage>,
}

impl AppDao {
    pub async fn new(
        user_account: Arc<UserAccount>,
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        redis: Arc<Mutex<ConnectionManager>>,
        time_out: u64,
    ) -> Result<AppDao, AppCoreError> {
        let app_locale_dir = app_core.app_dir.join("locale/lsys-app");
        let fluent = Arc::new(if app_locale_dir.exists() {
            app_core.create_fluent(app_locale_dir).await?
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            app_core
                .create_fluent(cargo_dir.to_owned() + "/locale")
                .await?
        });

        let app = Arc::from(Apps::new(
            app_core.clone(),
            db.clone(),
            redis.clone(),
            fluent.clone(),
        ));
        let app_oauth = Arc::from(AppsOauth::new(
            user_account,
            db.clone(),
            redis.clone(),
            fluent.clone(),
            time_out,
        ));
        Ok(AppDao {
            app,
            app_oauth,
            db,
            fluent,
            redis,
        })
    }
}
