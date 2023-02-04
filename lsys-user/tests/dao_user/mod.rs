use lsys_core::AppCore;
use lsys_user::dao::{auth::UserAuthRedisStore, UserDao};
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
mod account_dao;
async fn user_dao() -> UserDao<UserAuthRedisStore> {
    let app_core = AppCore::init("", &[]).await.unwrap();
    let db: Pool<MySql> = app_core.create_db().await.unwrap();
    let redis = Arc::new(Mutex::new(app_core.create_redis().await.unwrap()));
    let login_store = UserAuthRedisStore::new(redis.clone());
    let userdao = UserDao::new(Arc::new(app_core), db, redis, login_store, None)
        .await
        .unwrap();
    userdao
}
