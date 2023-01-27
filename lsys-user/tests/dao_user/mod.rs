// use lsys_core::AppCore;
// use lsys_user::dao::UserDao;
// use sqlx::{MySql, Pool};
// use std::sync::Arc;
// use tokio::sync::Mutex;

// #[cfg(test)]
// mod account_dao;
// async fn user_dao() -> UserDao {
//     let app_core = AppCore::init("", &[]).await.unwrap();
//     let db: Pool<MySql> = app_core.create_db().await.unwrap();
//     let redis = Mutex::new(app_core.create_redis().await.unwrap());
//     let userdao = UserDao::new(Arc::new(app_core), db, Arc::new(redis), None)
//         .await
//         .unwrap();
//     userdao
// }
