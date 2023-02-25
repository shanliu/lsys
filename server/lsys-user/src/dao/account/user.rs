use std::collections::HashMap;
use std::string::FromUtf8Error;
use std::sync::Arc;

use crate::dao::account::UserAccountResult;
use crate::model::{
    UserEmailModel, UserEmailModelRef, UserEmailStatus, UserIndexCat, UserIndexModel,
    UserIndexStatus, UserMobileModel, UserMobileModelRef, UserMobileStatus, UserModel,
    UserModelRef, UserNameModel, UserNameModelRef, UserStatus,
};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{get_message, now_time, FluentMessage, VecStringJoin};
use rand::prelude::SliceRandom;

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{model_option_set, Insert, ModelTableName, Select, Update};
use sqlx_model::{sql_format, SqlQuote};

use super::user_index::UserIndex;
use super::UserAccountError;

fn del_rand_name() -> Result<String, FromUtf8Error> {
    const BASE_STR: &str = "0123456789";
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, 6)
            .cloned()
            .collect(),
    )
}

pub struct User {
    db: Pool<MySql>,
    fluent: Arc<FluentMessage>,
    index: Arc<UserIndex>,
    pub cache: Arc<LocalCache<u64, UserModel>>,
}

pub struct FindUserParam {
    email: Option<String>,
    mobile: Option<String>,
    username: Option<String>,
    nikename: Option<String>,
    external_type: Option<String>,
    address_info: Option<String>,
    user_status: Option<UserStatus>,
}

// find_by_id_impl!(User,UserModel,cache,id,"");

impl User {
    pub fn new(
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        redis: deadpool_redis::Pool,
        index: Arc<UserIndex>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(redis, LocalCacheConfig::new("user"))),
            db,
            fluent,
            index,
        }
    }
    /// 添加用户
    pub async fn add_user<'t>(
        &self,
        nickname: String,
        status: UserStatus,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<UserModel> {
        if UserStatus::Init != status && UserStatus::Enable != status {
            return Err(UserAccountError::System(String::from(
                "submit status wrong",
            )));
        }
        let time = now_time()?;
        let u_status = status as i8;
        let new_data = model_option_set!(UserModelRef,{
            nickname:nickname,
            add_time:time,
            use_name:0,
            status:u_status,
        });
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
            .add(
                crate::model::UserIndexCat::UserStatus,
                user.id,
                &[user.status.to_string()],
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;
        Ok(user)
    }
    //激活用户
    pub async fn enable_user<'t>(
        &self,
        user: &UserModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<()> {
        if UserStatus::Delete.eq(user.status) {
            return Err(UserAccountError::System(String::from("user is delete")));
        }
        if UserStatus::Enable.eq(user.status) {
            return Ok(());
        }
        let change = sqlx_model::model_option_set!(UserModelRef,{
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
            .del(
                crate::model::UserIndexCat::UserStatus,
                user.id,
                &[user.status.to_string()],
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
                crate::model::UserIndexCat::UserStatus,
                user.id,
                &[(UserStatus::Enable as i8).to_string()],
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;
        Ok(())
    }
    //删除用户
    pub async fn del_user<'t>(
        &self,
        user: &UserModel,
        del_name: Option<String>,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<()> {
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let time = now_time()?;

        //change name is del_**
        let mut username = "del_".to_string() + time.to_string().as_str();
        if let Ok(rand) = del_rand_name() {
            username += rand.as_str();
        }
        let name_change = sqlx_model::model_option_set!(UserNameModelRef,{
            username:username,
            change_time:time
        });
        let res = Update::<sqlx::MySql, UserNameModel, _>::new(name_change)
            .execute_by_where_call("user_id=?", |e, _| e.bind(user.id), &mut db)
            .await;
        if let Err(e) = res {
            db.rollback().await?;
            return Err(e.into());
        }

        //delete all email
        let email_change = model_option_set!(UserEmailModelRef,{
            status:UserEmailStatus::Delete as i8,
            delete_time:time
        });
        let res = Update::<sqlx::MySql, UserEmailModel, _>::new(email_change)
            .execute_by_where_call("user_id=?", |e, _| e.bind(user.id), &mut db)
            .await;
        if let Err(e) = res {
            db.rollback().await?;
            return Err(e.into());
        }

        //delete all mobile
        let mobile_change = model_option_set!(UserMobileModelRef,{
            status:UserMobileStatus::Delete as i8,
            delete_time:time
        });
        let res = Update::<sqlx::MySql, UserMobileModel, _>::new(mobile_change)
            .execute_by_where_call("user_id=?", |e, _| e.bind(user.id), &mut db)
            .await;
        if let Err(e) = res {
            db.rollback().await?;
            return Err(e.into());
        }
        //delete index data
        if let Err(ie) = self.index.user_del(user.id, Some(&mut db)).await {
            db.rollback().await?;
            return Err(ie);
        }
        //delete user data
        let mut change = sqlx_model::model_option_set!(UserModelRef,{
            status:UserStatus::Delete as i8,
            delete_time:time
        });
        change.nickname = del_name.as_ref();
        let tmp = Update::<sqlx::MySql, UserModel, _>::new(change)
            .execute_by_pk(user, &mut db)
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e.into());
        }

        db.commit().await?;
        self.cache.clear(&user.id).await;
        Ok(())
    }
    pub async fn set_nikename<'t>(
        &self,
        user: &UserModel,
        nikename: String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let nikename = nikename.trim().to_string();
        if nikename.is_empty() || nikename.len() > 32 {
            return Err(UserAccountError::System(get_message!(
                &self.fluent,
                "user-nikename-wrong",
                "username length need 1-32 char"
            )));
        }
        let change = sqlx_model::model_option_set!(UserModelRef,{
            nickname:nikename,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, UserModel, _>::new(change)
            .execute_by_pk(user, &mut db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                if let Err(ie) = self
                    .index
                    .del(
                        crate::model::UserIndexCat::NikeName,
                        user.id,
                        &[user.nickname.to_owned()],
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
                        crate::model::UserIndexCat::NikeName,
                        user.id,
                        &[user.nickname.to_owned()],
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
    /// 通过内置索引查找用户数据
    pub async fn find_user(&self, param: &FindUserParam) -> UserAccountResult<Vec<i64>> {
        let mut where_sql = vec![];
        if let Some(ref tmp) = param.email {
            where_sql.push(sql_format!(
                "index_cat={} and index_data={}",
                UserIndexCat::Email as i8,
                tmp
            ));
        }
        if let Some(ref tmp) = param.mobile {
            where_sql.push(sql_format!(
                "index_cat={} and index_data={}",
                UserIndexCat::Mobile as i8,
                tmp
            ));
        }
        if let Some(ref tmp) = param.username {
            where_sql.push(sql_format!(
                "index_cat={} and index_data={}",
                UserIndexCat::UserName as i8,
                tmp
            ));
        }
        if let Some(ref tmp) = param.nikename {
            where_sql.push(sql_format!(
                "index_cat={} and index_data={}",
                UserIndexCat::NikeName as i8,
                tmp
            ));
        }
        if let Some(ref tmp) = param.external_type {
            where_sql.push(sql_format!(
                "index_cat={} and index_data={}",
                UserIndexCat::ExternalType as i8,
                tmp
            ));
        }
        if let Some(ref tmp) = param.address_info {
            where_sql.push(sql_format!(
                "index_cat={} and index_data={}",
                UserIndexCat::Address as i8,
                tmp
            ));
        }
        if let Some(tmp) = param.user_status {
            where_sql.push(sql_format!(
                "index_cat={} and index_data={}",
                UserIndexCat::UserStatus as i8,
                (tmp as i8).to_string()
            ));
        }
        let sql = if where_sql.is_empty() {
            sql_format!(
                "select user_id from {} order by id asc",
                UserModel::table_name(),
            )
        } else {
            sql_format!(
                "select user_id,count(*) as total from {} where status={} and {} group by user_id having total >={}",
                UserIndexModel::table_name(),
                UserIndexStatus::Enable as i8,
                where_sql.string_join(" and "),
                where_sql.len()
            )
        };
        let res = sqlx::query_scalar::<_, i64>(sql.as_str())
            .fetch_all(&self.db)
            .await?;
        Ok(res)
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
