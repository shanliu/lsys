use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    get_message, now_time, FluentMessage,
};

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{
    executor_option, model_option_set, sql_format, Insert, ModelTableName, Select, SqlQuote, Update,
};
use std::{collections::HashMap, sync::Arc};

use crate::model::{UserModel, UserNameModel, UserNameModelRef};

use super::{user_index::UserIndex, UserAccountError, UserAccountResult};

pub struct UserName {
    db: Pool<MySql>,
    fluent: Arc<FluentMessage>,
    index: Arc<UserIndex>,
    pub cache: Arc<LocalCache<u64, UserNameModel>>,
}

impl UserName {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        fluent: Arc<FluentMessage>,
        index: Arc<UserIndex>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(redis, LocalCacheConfig::new("user-name"))),
            db,
            fluent,
            index,
        }
    }
    /// 根据用户名查找记录
    pub async fn find_by_name(&self, name: String) -> UserAccountResult<UserNameModel> {
        let select = Select::type_new::<UserNameModel>();
        let res = select
            .fetch_one_by_where_call::<UserNameModel, _, _>(
                "username=?",
                |mut res, _| {
                    res = res.bind(name);
                    res
                },
                &self.db,
            )
            .await?;
        Ok(res)
    }
    /// 更改用户名
    pub async fn change_username<'t>(
        &self,
        user: &UserModel,
        username: String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<()> {
        let username = username.trim().to_string();
        if username.len() < 3 || username.len() > 32 {
            return Err(UserAccountError::System(get_message!(
                &self.fluent,
                "user-username-wrong",
                "username length need 3-32 char"
            )));
        }
        let time = now_time()?;
        let db = &self.db;
        let user_name_res = Select::type_new::<UserNameModel>()
            .fetch_one_by_where_call::<UserNameModel, _, _>(
                "username=?",
                |mut res, _| {
                    res = res.bind(username.clone());
                    res
                },
                db,
            )
            .await;
        match user_name_res {
            Err(sqlx::Error::RowNotFound) => {
                let user_name_res = Select::type_new::<UserNameModel>()
                    .fetch_one_by_where_call::<UserNameModel, _, _>(
                        "user_id=?",
                        |mut res, _| {
                            res = res.bind(user.id);
                            res
                        },
                        db,
                    )
                    .await;

                match user_name_res {
                    Err(sqlx::Error::RowNotFound) => {
                        let new_data = model_option_set!(UserNameModelRef,{
                            user_id:user.id,
                            username:username,
                            change_time: time,
                            add_time: time,
                        });

                        executor_option!(
                            {
                                Insert::<sqlx::MySql, UserNameModel, _>::new(new_data)
                                    .execute(db.as_copy())
                                    .await?;
                                sqlx::query(
                                    sql_format!(
                                        "UPDATE {} SET use_name=1 WHERE id=?",
                                        UserModel::table_name(),
                                    )
                                    .as_str(),
                                )
                                .bind(user.id)
                                .execute(db.as_copy())
                                .await?;
                            },
                            transaction,
                            db,
                            db
                        );

                        self.cache.clear(&user.id).await;
                        Ok(())
                    }
                    Ok(user_name) => {
                        let change = sqlx_model::model_option_set!(UserNameModelRef,{
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
                            .del(
                                crate::model::UserIndexCat::UserName,
                                user_name.user_id,
                                &[user_name.username.to_owned()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }
                        if let Err(ie) = self
                            .index
                            .add(
                                crate::model::UserIndexCat::UserName,
                                user_name.user_id,
                                &[username.clone()],
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
                        get_message!(&self.fluent,"user-name-exits","name {$name} already exists",["name"=>username]),
                    ))
                }
            }
            Err(err) => Err(err.into()),
        }
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
