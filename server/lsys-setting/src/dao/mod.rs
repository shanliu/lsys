mod setting;
mod setting_multiple;
mod setting_single;

use lsys_core::{AppCore, AppCoreError};
pub use setting::*;
pub use setting_multiple::*;
pub use setting_single::*;
use sqlx::{MySql, Pool};
use std::sync::Arc;

pub struct Setting {
    pub single: Arc<SingleSetting>,
    pub multiple: Arc<MultipleSetting>,
}

impl Setting {
    pub async fn new(
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
    ) -> Result<Self, AppCoreError> {
        let app_locale_dir = app_core.app_dir.join("locale/lsys-rbac");
        let fluents_message = Arc::new(if app_locale_dir.exists() {
            app_core.create_fluent(app_locale_dir).await?
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            app_core
                .create_fluent(cargo_dir.to_owned() + "/locale")
                .await?
        });
        Ok(Self {
            single: Arc::from(SingleSetting::new(
                db.clone(),
                fluents_message.clone(),
                redis.clone(),
            )),
            multiple: Arc::from(MultipleSetting::new(db, fluents_message, redis)),
        })
    }
}
