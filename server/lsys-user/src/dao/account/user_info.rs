use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    now_time, RemoteNotify, RequestEnv,
};

use lsys_logger::dao::ChangeLogger;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{sql_format, Insert, Select, Update};
use std::{collections::HashMap, sync::Arc};

use crate::model::{UserInfoModel, UserInfoModelRef, UserModel};

use super::{logger::LogUserInfo, user_index::UserIndex, UserAccountResult};
use sqlx_model::SqlQuote;
pub struct UserInfo {
    db: Pool<MySql>,

    index: Arc<UserIndex>,
    pub(crate) cache: Arc<LocalCache<u64, UserInfoModel>>,
    logger: Arc<ChangeLogger>,
}

//  find_by_id_impl!(UserInfo,UserInfoModel,cache,user_id,"");

impl UserInfo {
    pub fn new(
        db: Pool<MySql>,
      
        index: Arc<UserIndex>,
        remote_notify: Arc<RemoteNotify>,
        config:LocalCacheConfig,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            cache:Arc::new(LocalCache::new(remote_notify, config)),
            db,
            logger,
            index,
        }
    }
    /// 设置用户信息
    pub async fn set_info<'t, 't2>(
        &self,
        user: &UserModel,
        mut info: UserInfoModelRef<'t2>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<()> {
        let reg_from = info.reg_from.map(|e| e.to_string()).unwrap_or_default();
        let reg_ip = info.reg_ip.map(|e| e.to_string()).unwrap_or_default();
        let birthday = info.birthday.map(|e| e.to_string()).unwrap_or_default();
        let headimg = info.headimg.map(|e| e.to_string()).unwrap_or_default();
        let gender = info.gender.map(|e| e.to_owned()).unwrap_or_default();
        let time = now_time()?;
        info.user_id = Some(&user.id);
        info.change_time = Some(&time);
        let db = &self.db;
        let user_res = Select::type_new::<UserInfoModel>()
            .fetch_one_by_where::<UserInfoModel, _>(
                &sqlx_model::WhereOption::Where(sql_format!("user_id={}", user.id)),
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
                            .cat_one_add(
                                crate::model::UserIndexCat::RegFrom,
                                user.id,
                                rf.as_str(),
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

        self.logger
            .add(
                &LogUserInfo {
                    gender,
                    headimg,
                    birthday,
                    reg_ip,
                    reg_from,
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
