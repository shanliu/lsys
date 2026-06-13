use lsys_core::app_core::AppCore;
use lsys_core::app_core::utils;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::secret::{FieldEncryptor, SecretManager};
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::{SettingConfig, SettingDao};
use lsys_user::dao::{AccountConfig, AccountDao};
use sqlx::{MySql, Pool};
use std::sync::Arc;

#[cfg(test)]
mod account_dao;
#[allow(dead_code)]
async fn user_dao() -> AccountDao {
    let app_core: AppCore = AppCore::new("./", "config", "app", None, None).await.unwrap();
    let db: Pool<MySql> = utils::create_mysql_pool(&app_core).await.unwrap();
    let redis = utils::create_redis_pool(&app_core).await.unwrap();
    let app_core = Arc::new(app_core);
    let logger = Arc::new(ChangeLoggerDao::new(db.clone()));

    let remote_notify =
        Arc::new(RemoteNotify::new("lsys-remote-notify", app_core.clone(), redis.clone()).unwrap());
    let config = SettingDao::new(
        //   app_core.clone(),
        db.clone(),
        remote_notify.clone(),
        SettingConfig::new(false),
        logger.clone(),
    )
    .await
    .unwrap();
    
    let secret_manager = Arc::new(SecretManager::new());
    let field_encryptor = Arc::new(FieldEncryptor::new(
        secret_manager.clone(),
        "field_encrypt_key",
        true
    ));
    
    AccountDao::new(
        db,
        redis,
        config.single,
        AccountConfig::new(false),
        remote_notify,
        logger,
        secret_manager,
        field_encryptor,
    )
}
