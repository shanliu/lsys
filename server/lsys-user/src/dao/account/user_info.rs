use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    now_time,
};

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{Insert, Select,  Update};
use std::{collections::HashMap, sync::Arc};

use crate::model::{UserInfoModel, UserInfoModelRef, UserModel};

use super::{user_index::UserIndex, UserAccountResult};

pub struct UserInfo {
    db: Pool<MySql>,

    index: Arc<UserIndex>,
    pub cache: Arc<LocalCache<u64, UserInfoModel>>,
}

//  find_by_id_impl!(UserInfo,UserInfoModel,cache,user_id,"");

impl UserInfo {
    pub fn new(db: Pool<MySql>, redis: deadpool_redis::Pool, index: Arc<UserIndex>) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(redis, LocalCacheConfig::new("user-info"))),
            db,

            index,
        }
    }
    /// 设置用户信息
    pub async fn set_info<'t, 't2>(
        &self,
        user: &UserModel,
        mut info: UserInfoModelRef<'t2>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<()> {
        let time = now_time()?;
        info.user_id = Some(&user.id);
        info.change_time = Some(&time);
        let db = &self.db;
        let user_res = Select::type_new::<UserInfoModel>()
            .fetch_one_by_where_call::<UserInfoModel, _, _>(
                "user_id=?",
                |mut res, _| {
                    res = res.bind(user.id);
                    res
                },
                db,
            )
            .await;
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let tmp = match user_res {
            Err(sqlx::Error::RowNotFound) => {
                if let Some(rf) = info.reg_from {
                    if !rf.is_empty() {
                        if let Err(ie) = self
                            .index
                            .add(
                                crate::model::UserIndexCat::RegFrom,
                                user.id,
                                &[rf.to_owned()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }
                    }
                }

                Insert::<sqlx::MySql, UserInfoModel, _>::new(info)
                    .execute(&mut db)
                    .await
            }
            Ok(user_info) => {
                if !user_info.reg_from.is_empty() {
                    if let Err(ie) = self
                        .index
                        .del(
                            crate::model::UserIndexCat::RegFrom,
                            user.id,
                            &[user_info.reg_from.to_owned()],
                            Some(&mut db),
                        )
                        .await
                    {
                        db.rollback().await?;
                        return Err(ie);
                    }
                }
                if let Some(rf) = info.reg_from {
                    if !rf.is_empty() {
                        if let Err(ie) = self
                            .index
                            .add(
                                crate::model::UserIndexCat::RegFrom,
                                user.id,
                                &[rf.to_owned()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }
                    }
                }

                Update::<sqlx::MySql, UserInfoModel, _>::new(info)
                    .execute_by_pk(&user_info, &mut db)
                    .await
            }
            Err(err) => {
                return Err(err.into());
            }
        };
        if let Err(ie) = tmp {
            db.rollback().await?;
            return Err(ie.into());
        };
        db.commit().await?;
        self.cache.clear(&user.id).await;
        Ok(())
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_user_id,
        u64,
        UserInfoModel,
        UserAccountResult<UserInfoModel>,
        id,
        "user_id = {id}"
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_user_ids,
        u64,
        UserInfoModel,
        UserAccountResult<HashMap<u64, UserInfoModel>>,
        user_id,
        ids,
        "user_id in ({ids})"
    );
    pub fn cache(&'_ self) -> UserInfoCache<'_> {
        UserInfoCache { dao: self }
    }
}

pub struct UserInfoCache<'t> {
    pub dao: &'t UserInfo,
}
impl<'t> UserInfoCache<'t> {
    lsys_core::impl_cache_fetch_one!(
        find_by_user_id,
        dao,
        cache,
        u64,
        UserAccountResult<UserInfoModel>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_user_ids,
        dao,
        cache,
        u64,
        UserAccountResult<HashMap<u64, UserInfoModel>>
    );
}
