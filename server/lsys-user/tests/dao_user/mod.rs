use lsys_core::AppCore;
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
    let login_store = UserAuthRedisStore::new(redis.clone());
    UserDao::new(Arc::new(app_core), db, redis, login_store, None)
        .await
        .unwrap()
}
