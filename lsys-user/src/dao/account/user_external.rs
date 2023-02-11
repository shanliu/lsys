use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::account::{UserAccountError, UserAccountResult};

use crate::model::{UserExternalModel, UserExternalModelRef, UserExternalStatus, UserModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{get_message, now_time, FluentMessage};

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{
    executor_option, model_option_set, sql_format, Insert, ModelTableName, Select, SqlQuote, Update,
};

use super::user_index::UserIndex;

pub struct UserExternal {
    db: Pool<MySql>,
    index: Arc<UserIndex>,
    fluent: Arc<FluentMessage>,
    pub cache: Arc<LocalCache<u64, Vec<UserExternalModel>>>,
}

impl UserExternal {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        fluent: Arc<FluentMessage>,
        index: Arc<UserIndex>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                redis,
                LocalCacheConfig::new("user-external"),
            )),
            db,
            fluent,
            index,
        }
    }

    /// 根据第三方信息查找记录
    pub async fn find_by_external(
        &self,
        config_name: &String,
        external_type: &String,
        external_id: &String,
    ) -> UserAccountResult<UserExternalModel> {
        let select = Select::type_new::<UserExternalModel>();
        let res = select
            .fetch_one_by_where_call::<UserExternalModel, _, _>(
                "config_name=? and external_type=? and external_id=? and status=? order by id desc",
                |mut res, _| {
                    res = res.bind(config_name.to_owned());
                    res = res.bind(external_type.to_owned());
                    res = res.bind(external_id.to_owned());
                    res = res.bind(UserExternalStatus::Enable as i8);
                    res
                },
                &self.db,
            )
            .await?;
        Ok(res)
    }
    /// 根据用户跟第三方id查找记录
    pub async fn find_by_user_external(
        &self,
        user: &UserModel,
        config_name: String,
        external_type: String,
        external_id: String,
    ) -> UserAccountResult<UserExternalModel> {
        let select = Select::type_new::<UserExternalModel>();
        let res = select
            .fetch_one_by_where_call::<UserExternalModel, _, _>(
                "user_id=? and config_name=? and external_type=? and external_id=? and status = ? order by id desc",
                | res, _| {
                   res.bind(user.id)
                    .bind(config_name)
                    .bind(external_type)
                    .bind(external_id)
                    .bind(UserExternalStatus::Enable as i8)
                },
                &self.db,
            )
            .await?;
        Ok(res)
    }
    /// 新增第三方登录信息
    pub async fn add_external<'t>(
        &self,
        user: &UserModel,
        config_name: String,
        external_type: String,
        external_id: String,
        external_name: String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let db = &self.db;
        let user_ext_res = Select::type_new::<UserExternalModel>()
            .fetch_one_by_where_call::<UserExternalModel, _, _>(
                "config_name=? and  external_type=? and external_id=? and status = ?",
                |res, _| {
                    res.bind(config_name.clone())
                        .bind(external_type.clone())
                        .bind(external_id.clone())
                        .bind(UserExternalStatus::Enable as i8)
                },
                db,
            )
            .await;
        let time = now_time()?;
        match user_ext_res {
            Ok(user_ext) => {
                if user_ext.user_id != user.id {
                    return Err(UserAccountError::System(get_message!(&self.fluent,
                        "user-external-other-bind","this account {$name} bind in other account[{$id}]",
                        ["name"=>external_name,"id"=>user.id ]
                    )));
                }
                let change = sqlx_model::model_option_set!(UserExternalModelRef,{
                    status:UserExternalStatus::Enable as i8,
                    external_name:external_name,
                    add_time:time
                });
                executor_option!(
                    {
                        Update::<sqlx::MySql, UserExternalModel, _>::new(change)
                            .execute_by_pk(&user_ext, db)
                            .await?;
                    },
                    transaction,
                    db,
                    db
                );
                Ok(user_ext.id)
            }
            Err(sqlx::Error::RowNotFound) => {
                let new_data = model_option_set!(UserExternalModelRef,{
                    user_id:user.id,
                    status:UserExternalStatus::Enable as i8,
                    config_name:config_name,
                    external_type:external_type,
                    external_id:external_id,
                    external_name:external_name,
                    change_time:time,
                    add_time:time
                });

                let mut db = match transaction {
                    Some(pb) => pb.begin().await?,
                    None => db.begin().await?,
                };
                let res = Insert::<sqlx::MySql, UserExternalModel, _>::new(new_data)
                    .execute(&mut db)
                    .await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        Err(e)?
                    }
                    Ok(mr) => {
                        let res = sqlx::query(
                            sql_format!(
                                "UPDATE {} SET external_count=external_count+1 WHERE id=?",
                                UserModel::table_name(),
                            )
                            .as_str(),
                        )
                        .bind(user.id)
                        .execute(&mut db)
                        .await;
                        match res {
                            Err(e) => {
                                db.rollback().await?;
                                Err(e.into())
                            }
                            Ok(_) => {
                                if let Err(ie) = self
                                    .index
                                    .add(
                                        crate::model::UserIndexCat::ExternalType,
                                        user.id,
                                        &[external_type],
                                        Some(&mut db),
                                    )
                                    .await
                                {
                                    db.rollback().await?;
                                    return Err(ie);
                                }

                                db.commit().await?;
                                self.cache.clear(&user.id).await;
                                Ok(mr.last_insert_id())
                            }
                        }
                    }
                }
            }
            Err(err) => Err(err)?,
        }
    }
    /// 刷新第三方登录token
    #[allow(clippy::too_many_arguments)]
    pub async fn token_update(
        &self,
        user_ext: &UserExternalModel,
        external_name: String,
        token_data: String,
        token_timeout: u64,
        external_nikename: Option<String>,
        external_gender: Option<String>,
        external_link: Option<String>,
        external_pic: Option<String>,
    ) -> UserAccountResult<()> {
        let time = now_time()?;
        let mut change = sqlx_model::model_option_set!(UserExternalModelRef,{
            external_name:external_name,
            token_data:token_data,
            token_timeout:token_timeout,
            change_time:time,
        });
        change.external_link = external_link.as_ref();
        change.external_gender = external_gender.as_ref();
        change.external_pic = external_pic.as_ref();
        change.external_nikename = external_nikename.as_ref();
        Update::<sqlx::MySql, UserExternalModel, _>::new(change)
            .execute_by_pk(user_ext, &self.db)
            .await?;
        self.cache.clear(&user_ext.user_id).await;
        Ok(())
    }
    /// 删除用户外部账号
    pub async fn del_external<'t>(
        &self,
        user_ext: &UserExternalModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserExternalModelRef,{
            status:UserExternalStatus::Delete as i8,
            delete_time:time
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, UserExternalModel, _>::new(change)
            .execute_by_pk(user_ext, &mut db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res=sqlx::query(sql_format!(
                        "UPDATE {} SET external_count=external_count-1 WHERE id=? and external_count-1>=0",
                        UserModel::table_name(),
                    ).as_str())
                    .bind(user_ext.user_id)
                    .execute(&mut db).await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        Err(e.into())
                    }
                    Ok(_) => {
                        if let Err(ie) = self
                            .index
                            .del(
                                crate::model::UserIndexCat::ExternalType,
                                user_ext.user_id,
                                &[user_ext.external_type.clone()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&user_ext.user_id).await;
                        Ok(mr.rows_affected())
                    }
                }
            }
        }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        UserExternalModel,
        UserAccountResult<UserExternalModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_user_id_vec,
        u64,
        UserExternalModel,
        UserAccountResult<Vec<UserExternalModel>>,
        uid,
        "user_id = {uid} and status = {status}",
        status = UserExternalStatus::Enable as i8
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_user_ids_vec,
        u64,
        UserExternalModel,
        UserAccountResult<HashMap<u64, Vec<UserExternalModel>>>,
        user_id,
        uid,
        "user_id in ({uid}) and status = {status}",
        status = UserExternalStatus::Enable as i8
    );
    pub fn cache(&'_ self) -> UserExternalCache<'_> {
        UserExternalCache { dao: self }
    }
}

pub struct UserExternalCache<'t> {
    pub dao: &'t UserExternal,
}
impl<'t> UserExternalCache<'t> {
    lsys_core::impl_cache_fetch_one!(
        find_by_user_id_vec,
        dao,
        cache,
        u64,
        UserAccountResult<Vec<UserExternalModel>>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_user_ids_vec,
        dao,
        cache,
        u64,
        UserAccountResult<HashMap<u64, Vec<UserExternalModel>>>
    );
}
