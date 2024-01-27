use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    fluent_message, now_time, rand_str, RandType, RemoteNotify, RequestEnv,
};

use lsys_logger::dao::ChangeLogger;
// use rand::seq::SliceRandom;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{
    model_option_set, sql_format, Insert, ModelTableName, Select, SqlQuote, Update, WhereOption,
};
use std::{collections::HashMap, string::FromUtf8Error, sync::Arc};

use crate::model::{UserModel, UserNameModel, UserNameModelRef, UserNameStatus};

use super::{logger::LogUserName, user_index::UserIndex, UserAccountError, UserAccountResult};

pub struct UserName {
    db: Pool<MySql>,
    // fluent: Arc<FluentBuild>,
    index: Arc<UserIndex>,
    pub(crate) cache: Arc<LocalCache<u64, UserNameModel>>,
    logger: Arc<ChangeLogger>,
}

fn del_rand_name() -> Result<String, FromUtf8Error> {
    Ok(rand_str(RandType::LowerHex, 6))
    // const BASE_STR: &str = "0123456789";
    // let mut rng = &mut rand::thread_rng();
    // String::from_utf8(
    //     BASE_STR
    //         .as_bytes()
    //         .choose_multiple(&mut rng, 6)
    //         .cloned()
    //         .collect(),
    // )
}

impl UserName {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        //    fluent: Arc<FluentBuild>,
        index: Arc<UserIndex>,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                remote_notify,
                LocalCacheConfig::new("user-name"),
            )),
            db,
            //    fluent,
            index,
            logger,
        }
    }
    /// 根据用户名查找记录
    pub async fn find_by_name(&self, name: String) -> UserAccountResult<UserNameModel> {
        let select = Select::type_new::<UserNameModel>();
        let res = select
            .fetch_one_by_where::<UserNameModel, _>(
                &WhereOption::Where(sql_format!(
                    "username={} and status={}",
                    name,
                    UserNameStatus::Enable
                )),
                &self.db,
            )
            .await?;
        Ok(res)
    }
    /// 移除用户登录
    pub async fn remove_username<'t>(
        &self,
        user: &UserModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<()> {
        //change name is del_**
        let ntime = now_time().unwrap_or_default();
        let mut username = "delete_".to_string() + ntime.to_string().as_str();
        if let Ok(rand) = del_rand_name() {
            username += rand.as_str();
        }
        let status = UserNameStatus::Delete as i8;
        let name_change = sqlx_model::model_option_set!(UserNameModelRef,{
            username:username,
            change_time:ntime,
            status:status
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, UserNameModel, _>::new(name_change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("user_id={}", user.id)),
                &mut db,
            )
            .await;
        if let Err(e) = res {
            db.rollback().await?;
            return Err(e.into());
        }
        if let Err(ie) = self
            .index
            .cat_del(crate::model::UserIndexCat::UserName, user.id, Some(&mut db))
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;

        self.logger
            .add(
                &LogUserName {
                    action: "del",
                    username,
                },
                &Some(user.id),
                &Some(user.id),
                &Some(user.id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    /// 更改用户名
    pub async fn change_username<'t>(
        &self,
        user: &UserModel,
        username: String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<()> {
        let username = username.trim().to_string();
        if username.len() < 3 || username.len() > 32 || username.starts_with("delete_") {
            return Err(UserAccountError::System(
                fluent_message!("user-username-error",
                    {
                        "len":username.len(),
                        "min":3,
                        "max":32,
                        "bad_start":"delete_"
                    }
                ),
            )); //"username length need 3-32 char and username can't start [delete_]"
        }
        let time = now_time()?;
        let db = &self.db;
        let user_name_res = Select::type_new::<UserNameModel>()
            .fetch_one_by_where::<UserNameModel, _>(
                &WhereOption::Where(sql_format!("username={}", username)),
                db,
            )
            .await;
        let out = match user_name_res {
            Err(sqlx::Error::RowNotFound) => {
                let user_name_res = Select::type_new::<UserNameModel>()
                    .fetch_one_by_where::<UserNameModel, _>(
                        &WhereOption::Where(sql_format!("user_id={}", user.id)),
                        db,
                    )
                    .await;
                match user_name_res {
                    Err(sqlx::Error::RowNotFound) => {
                        let status = UserNameStatus::Enable as i8;
                        let new_data = model_option_set!(UserNameModelRef,{
                            user_id:user.id,
                            username:username,
                            status:status,
                            change_time: time,
                        });
                        let mut db = match transaction {
                            Some(pb) => pb.begin().await?,
                            None => self.db.begin().await?,
                        };
                        let tmp = Insert::<sqlx::MySql, UserNameModel, _>::new(new_data)
                            .execute(&mut db)
                            .await;
                        if let Err(ie) = tmp {
                            db.rollback().await?;
                            return Err(ie.into());
                        }
                        let tmp = sqlx::query(
                            sql_format!(
                                "UPDATE {} SET use_name=1 WHERE id=?",
                                UserModel::table_name(),
                            )
                            .as_str(),
                        )
                        .bind(user.id)
                        .execute(&mut db)
                        .await;
                        if let Err(ie) = tmp {
                            db.rollback().await?;
                            return Err(ie.into());
                        }
                        if let Err(ie) = self
                            .index
                            .cat_one_add(
                                crate::model::UserIndexCat::UserName,
                                user.id,
                                &username,
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }
                        db.commit().await?;
                        self.cache.clear(&user.id).await;
                        Ok(())
                    }
                    Ok(user_name) => {
                        let status = UserNameStatus::Enable as i8;
                        let change = sqlx_model::model_option_set!(UserNameModelRef,{
                            status:status,
                            username:username,
                            change_time:time
                        });
                        let mut db = match transaction {
                            Some(pb) => pb.begin().await?,
                            None => self.db.begin().await?,
                        };
                        let tmp = Update::<sqlx::MySql, UserNameModel, _>::new(change)
                            .execute_by_pk(&user_name, &mut db)
                            .await;
                        if let Err(ie) = tmp {
                            db.rollback().await?;
                            return Err(ie.into());
                        }
                        if let Err(ie) = self
                            .index
                            .cat_one_add(
                                crate::model::UserIndexCat::UserName,
                                user_name.user_id,
                                &username,
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }
                        db.commit().await?;
                        self.cache.clear(&user.id).await;
                        Ok(())
                    }
                    Err(err) => Err(err.into()),
                }
            }
            Ok(user_name) => {
                if user_name.user_id == user.id {
                    Ok(())
                } else {
                    Err(UserAccountError::System(
                        fluent_message!("user-name-exits",{"name":username}), //"name {$name} already exists",
                    ))
                }
            }
            Err(err) => Err(err.into()),
        };
        if out.is_ok() {
            self.logger
                .add(
                    &LogUserName {
                        action: "set",
                        username,
                    },
                    &Some(user.id),
                    &Some(user.id),
                    &Some(user.id),
                    None,
                    env_data,
                )
                .await;
        }
        out
    }

    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_user_id,
        u64,
        UserNameModel,
        UserAccountResult<UserNameModel>,
        id,
        "user_id = {id}  order by id desc"
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_user_ids,
        u64,
        UserNameModel,
        UserAccountResult<HashMap<u64, UserNameModel>>,
        user_id,
        ids,
        "user_id in ({ids})  order by id desc"
    );
    pub fn cache(&'_ self) -> UserNameCache<'_> {
        UserNameCache { dao: self }
    }
}

pub struct UserNameCache<'t> {
    pub dao: &'t UserName,
}
impl<'t> UserNameCache<'t> {
    lsys_core::impl_cache_fetch_one!(
        find_by_user_id,
        dao,
        cache,
        u64,
        UserAccountResult<UserNameModel>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_user_ids,
        dao,
        cache,
        u64,
        UserAccountResult<HashMap<u64, UserNameModel>>
    );
}
