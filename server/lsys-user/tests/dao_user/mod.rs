use lsys_core::{AppCore, RemoteNotify};
use lsys_logger::dao::ChangeLogger;
use lsys_setting::dao::Setting;
use lsys_user::dao::{auth::UserAuthRedisStore, UserDao};
use sqlx::{MySql, Pool};
use std::sync::Arc;

#[cfg(test)]
mod account_dao;
#[allow(dead_code)]
async fn user_dao() -> UserDao<UserAuthRedisStore> {
    let app_core = AppCore::init("", &[]).await.unwrap();
    let db: Pool<MySql> = app_core.create_db().await.unwrap();
    let redis = app_core.create_redis().await.unwrap();
    let app_core = Arc::new(app_core);
    let logger = Arc::new(ChangeLogger::new(db.clone()));
    let remote_notify =
        Arc::new(RemoteNotify::new("lsys-remote-notify", app_core.clone(), redis.clone()).unwrap());
    let config = Setting::new(
        app_core.clone(),
        db.clone(),
        remote_notify.clone(),
        logger.clone(),
    )
    .await
    .unwrap();
    let login_store = UserAuthRedisStore::new(redis.clone());
    UserDao::new(
        app_core,
        db,
        redis,
        config.single,
        logger,
        remote_notify,
        login_store,
        None,
    )
    .await
    .unwrap()
}
