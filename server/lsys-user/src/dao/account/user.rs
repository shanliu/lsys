use std::collections::HashMap;

use std::sync::Arc;

use crate::dao::account::UserAccountResult;
use crate::model::{UserIndexCat, UserModel, UserModelRef, UserStatus};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{get_message, now_time, FluentMessage, LimitParam, RemoteNotify, RequestEnv};
use lsys_logger::dao::ChangeLogger;

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{model_option_set, Insert, Select, Update};

use super::logger::LogUser;
use super::user_index::{UserIndex, UserItem};
use super::UserAccountError;

pub struct User {
    db: Pool<MySql>,
    fluent: Arc<FluentMessage>,
    index: Arc<UserIndex>,
    pub cache: Arc<LocalCache<u64, UserModel>>,
    logger: Arc<ChangeLogger>,
}

// find_by_id_impl!(User,UserModel,cache,id,"");

impl User {
    pub fn new(
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        remote_notify: Arc<RemoteNotify>,
        index: Arc<UserIndex>,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                remote_notify,
                LocalCacheConfig::new("user"),
            )),
            db,
            fluent,
            index,
            logger,
        }
    }
    /// 添加用户
    pub async fn add_user<'t>(
        &self,
        nickname: String,
        status: UserStatus,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<UserModel> {
        if UserStatus::Init != status && UserStatus::Enable != status {
            return Err(UserAccountError::System(String::from(
                "submit status wrong",
            )));
        }
        let time = now_time()?;
        let u_status = status as i8;
        let mut new_data = model_option_set!(UserModelRef,{
            nickname:nickname,
            add_time:time,
            change_time:time,
            use_name:0,
            status:u_status,
        });

        if UserStatus::Enable == status {
            new_data.confirm_time = Some(&time);
        }

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Insert::<sqlx::MySql, UserModel, _>::new(new_data)
            .execute(&mut db)
            .await;
        let res = match tmp {
            Ok(e) => e,
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
        };
        let user_id = res.last_insert_id();
        let tmp = Select::type_new::<UserModel>()
            .fetch_one_by_where_call::<UserModel, _, _>(
                "id=?",
                |mut res, _| {
                    res = res.bind(user_id);
                    res
                },
                &mut db,
            )
            .await;
        let user = match tmp {
            Ok(e) => e,
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
        };
        if let Err(ie) = self
            .index
            .cat_one_add(
                crate::model::UserIndexCat::UserStatus,
                user.id,
                &user.status.to_string(),
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;

        self.logger
            .add(
                &LogUser {
                    action: "add",
                    nickname,
                    status: u_status,
                },
                &Some(user.id),
                &Some(user.id),
                &Some(user.id),
                None,
                env_data,
            )
            .await;

        Ok(user)
    }
    //激活用户
    pub async fn enable_user<'t>(
        &self,
        user: &UserModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<()> {
        if UserStatus::Delete.eq(user.status) {
            return Err(UserAccountError::System(String::from("user is delete")));
        }
        if UserStatus::Enable.eq(user.status) {
            return Ok(());
        }
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(UserModelRef,{
            change_time:time,
            confirm_time:time,
            status:UserStatus::Enable as i8,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<sqlx::MySql, UserModel, _>::new(change)
            .execute_by_pk(user, &mut db)
            .await;
        if let Err(ie) = tmp {
            db.rollback().await?;
            return Err(ie.into());
        }
        if let Err(ie) = self
            .index
            .cat_one_add(
                crate::model::UserIndexCat::UserStatus,
                user.id,
                &(UserStatus::Enable as i8).to_string(),
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;

        self.logger
            .add(
                &LogUser {
                    action: "enable",
                    nickname: user.nickname.to_owned(),
                    status: UserStatus::Enable as i8,
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
    //删除用户
    pub async fn del_user<'t>(
        &self,
        user: &UserModel,
        del_name: Option<String>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<()> {
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let time = now_time()?;
        //delete user data
        let mut change = sqlx_model::model_option_set!(UserModelRef,{
            status:UserStatus::Delete as i8,
            change_time:time
        });
        change.nickname = del_name.as_ref();
        let tmp = Update::<sqlx::MySql, UserModel, _>::new(change)
            .execute_by_pk(user, &mut db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e.into());
        }
        if let Err(ie) = self.index.user_del(user.id, Some(&mut db)).await {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;
        self.cache.clear(&user.id).await;
        self.logger
            .add(
                &LogUser {
                    action: "del",
                    nickname: user.nickname.to_owned(),
                    status: UserStatus::Delete as i8,
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
    pub async fn set_nikename<'t>(
        &self,
        user: &UserModel,
        nikename: String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<u64> {
        let nikename = nikename.trim().to_string();
        if nikename.is_empty() || nikename.len() > 32 {
            return Err(UserAccountError::System(get_message!(
                &self.fluent,
                "user-nikename-wrong",
                "username length need 1-32 char"
            )));
        }
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(UserModelRef,{
            change_time:time,
            nickname:nikename,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, UserModel, _>::new(change)
            .execute_by_pk(user, &mut db)
            .await;
        let out = match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                if let Err(ie) = self
                    .index
                    .cat_one_add(
                        crate::model::UserIndexCat::NikeName,
                        user.id,
                        &user.nickname,
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
        };
        self.logger
            .add(
                &LogUser {
                    action: "nikename",
                    nickname: user.nickname.to_owned(),
                    status: user.status,
                },
                &Some(user.id),
                &Some(user.id),
                &Some(user.id),
                None,
                env_data,
            )
            .await;
        out
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        UserModel,
        UserAccountResult<UserModel>,
        id,
        "id = {id} and status in ({status})",
        status = [UserStatus::Enable as i8, UserStatus::Init as i8]
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        UserModel,
        UserAccountResult<HashMap<u64, UserModel>>,
        id,
        ids,
        "id in ({ids}) and status in ({status})",
        status = [UserStatus::Enable as i8, UserStatus::Init as i8]
    );
    //搜索用户
    pub async fn search_user(
        &self,
        key_word: &str,
        enable_user: bool,
        limit: &Option<LimitParam>,
    ) -> UserAccountResult<(Vec<UserItem>, Option<u64>)> {
        self.index
            .search_user(
                if enable_user {
                    &[UserStatus::Enable]
                } else {
                    &[UserStatus::Enable, UserStatus::Init]
                },
                key_word,
                &[
                    UserIndexCat::NikeName,
                    UserIndexCat::UserName,
                    UserIndexCat::Email,
                    UserIndexCat::Mobile,
                ],
                limit,
            )
            .await
    }

    pub fn cache(&'_ self) -> UserCache<'_> {
        UserCache { dao: self }
    }
}

pub struct UserCache<'t> {
    pub dao: &'t User,
}
impl<'t> UserCache<'t> {
    lsys_core::impl_cache_fetch_one!(find_by_id, dao, cache, u64, UserAccountResult<UserModel>);
    lsys_core::impl_cache_fetch_vec!(
        find_by_ids,
        dao,
        cache,
        u64,
        UserAccountResult<HashMap<u64, UserModel>>
    );
}
