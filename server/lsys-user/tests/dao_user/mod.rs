use lsys_access::dao::{AccessConfig, AccessDao};
use lsys_core::{AppCore, RemoteNotify};
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::{SettingConfig, SettingDao};
use lsys_user::dao::{AccountConfig, AccountDao};
use sqlx::{MySql, Pool};
use std::sync::Arc;

#[cfg(test)]
mod account_dao;
#[allow(dead_code)]
async fn user_dao() -> AccountDao {
    let app_core = AppCore::init("", "config", None, None).await.unwrap();
    let db: Pool<MySql> = app_core.create_db().await.unwrap();
    let redis = app_core.create_redis().await.unwrap();
    let app_core = Arc::new(app_core);
    let logger = Arc::new(ChangeLoggerDao::new(db.clone()));

    let remote_notify =
        Arc::new(RemoteNotify::new("lsys-remote-notify", app_core.clone(), redis.clone()).unwrap());
    let access = Arc::new(AccessDao::new(
        db.clone(),
        redis.clone(),
        remote_notify.clone(),
        AccessConfig::new(false),
    ));
    let config = SettingDao::new(
        //   app_core.clone(),
        db.clone(),
        remote_notify.clone(),
        SettingConfig::new(false),
        logger.clone(),
    )
    .await
    .unwrap();
    AccountDao::new(
        db,
        redis,
        config.single,
        access,
        AccountConfig::new(false),
        remote_notify,
        logger,
    )
}
