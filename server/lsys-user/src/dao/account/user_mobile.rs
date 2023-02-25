use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::account::UserAccountResult;

use crate::model::{UserMobileModel, UserMobileModelRef, UserMobileStatus, UserModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::FluentMessage;
use lsys_core::{get_message, now_time};

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, Select, SqlQuote, Update};

use tracing::log::warn;

use super::user_index::UserIndex;
use super::{check_mobile, UserAccountError};
pub trait UserMobileValid {
    fn check(mobile: String) -> UserAccountResult<bool>;
}

pub struct UserMobile {
    db: Pool<MySql>,
    redis: deadpool_redis::Pool,
    fluent: Arc<FluentMessage>,
    index: Arc<UserIndex>,
    pub cache: Arc<LocalCache<u64, Vec<UserMobileModel>>>,
}

impl UserMobile {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        fluent: Arc<FluentMessage>,
        index: Arc<UserIndex>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                redis.clone(),
                LocalCacheConfig::new("user-mobile"),
            )),
            db,
            redis,
            fluent,
            index,
        }
    }

    /// 通过手机号查找用户手机号记录
    pub async fn find_by_last_mobile(
        &self,
        area_code: String,
        mobile: String,
    ) -> UserAccountResult<UserMobileModel> {
        let select = Select::type_new::<UserMobileModel>();
        let res = select
            .fetch_one_by_where_call::<UserMobileModel, _, _>(
                "mobile=? and area_code=?  and status in (?,?) order by id desc",
                |mut res, _| {
                    res = res.bind(mobile);
                    res = res.bind(area_code);
                    res = res.bind(UserMobileStatus::Init as i8);
                    res = res.bind(UserMobileStatus::Valid as i8);
                    res
                },
                &self.db,
            )
            .await?;
        Ok(res)
    }
    /// 添加手机号
    pub async fn add_mobile<'t>(
        &self,
        user: &UserModel,
        area_code: String,
        mobile: String,
        mut status: UserMobileStatus,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        if status == UserMobileStatus::Delete {
            status = UserMobileStatus::Init;
        }
        check_mobile(&self.fluent, area_code.as_str(), mobile.as_str())?;
        let mobile_res = Select::type_new::<UserMobileModel>()
            .fetch_one_by_where_call::<UserMobileModel, _, _>(
                "area_code=? and mobile=? and status in (?,?)",
                |mut res, _| {
                    res = res.bind(area_code.clone());
                    res = res.bind(mobile.clone());
                    res = res.bind(UserMobileStatus::Valid as i8);
                    res = res.bind(UserMobileStatus::Init as i8);
                    res
                },
                &self.db,
            )
            .await;
        match mobile_res {
            Ok(mobile) => {
                if mobile.user_id == user.id {
                    return Ok(mobile.id);
                } else {
                    return Err(UserAccountError::System(get_message!(&self.fluent,
                        "user-mobile-exits","mobile {$name} bind on other account[{$id}]",
                        ["name"=>mobile.mobile,"id"=>mobile.user_id ]
                    )));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }

        let time = now_time()?;
        let _status = status as i8;
        let mut idata = model_option_set!(UserMobileModelRef,{
            mobile:mobile,
            status:_status,
            area_code:area_code,
            user_id:user.id,
            add_time:time,
        });

        if status == UserMobileStatus::Valid {
            idata.confirm_time = Some(&time);
        }

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<sqlx::MySql, UserMobileModel, _>::new(idata)
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
                        "UPDATE {} SET mobile_count=mobile_count+1 WHERE id=?",
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
                        if UserMobileStatus::Valid == status {
                            if let Err(ie) = self
                                .index
                                .add(
                                    crate::model::UserIndexCat::Mobile,
                                    user.id,
                                    &[mobile],
                                    Some(&mut db),
                                )
                                .await
                            {
                                db.rollback().await?;
                                return Err(ie);
                            }
                        }

                        db.commit().await?;
                        self.cache.clear(&user.id).await;
                        Ok(mr.last_insert_id())
                    }
                }
            }
        }
    }
    impl_account_valid_code_method!("mobile",{
        area_code: &String,
        mobile: &str,
    },{
        area_code.to_owned() + mobile
    },120);
    /// 验证code并确认手机号
    pub async fn confirm_mobile_from_code(
        &self,
        user_mobile: &UserMobileModel,
        code: &String,
    ) -> UserAccountResult<u64> {
        self.valid_code_check(code, &user_mobile.area_code, &user_mobile.mobile)
            .await?;
        let res = self.confirm_mobile(user_mobile).await;
        if res.is_ok() {
            if let Err(err) = self
                .valid_code_clear(&user_mobile.area_code, &user_mobile.mobile)
                .await
            {
                warn!(
                    "mobile {}-{} valid clear fail:{}",
                    &user_mobile.area_code, &user_mobile.mobile, err
                );
            }
        }
        res
    }
    //确认手机号
    pub async fn confirm_mobile(&self, user_mobile: &UserMobileModel) -> UserAccountResult<u64> {
        let mobile_res = Select::type_new::<UserMobileModel>()
            .fetch_one_by_where_call::<UserMobileModel, _, _>(
                " area_code=? and mobile=? and status =? and user_id!=? and id!=?",
                |mut res, _| {
                    res = res.bind(user_mobile.area_code.clone());
                    res = res.bind(user_mobile.mobile.clone());
                    res = res.bind(UserMobileStatus::Valid as i8);
                    res = res.bind(user_mobile.user_id);
                    res = res.bind(user_mobile.id);
                    res
                },
                &self.db,
            )
            .await;
        match mobile_res {
            Ok(mobile) => {
                return Err(UserAccountError::System(get_message!(&self.fluent,
                    "user-mobile-exits","confirm error : mobile {$name} bind on other account[{$id}]",
                    ["name"=>mobile.mobile,"id"=>mobile.user_id ]
                )));
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserMobileModelRef,{
            status:UserMobileStatus::Valid as i8,
            confirm_time:time
        });
        let mut db = self.db.begin().await?;

        let tmp = Update::<sqlx::MySql, UserMobileModel, _>::new(change)
            .execute_by_pk(user_mobile, &mut db)
            .await;
        let res = match tmp {
            Ok(e) => e,
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
        };
        if let Err(ie) = self
            .index
            .add(
                crate::model::UserIndexCat::Mobile,
                user_mobile.user_id,
                &[user_mobile.mobile.to_owned()],
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;
        self.cache.clear(&user_mobile.user_id).await;
        Ok(res.rows_affected())
    }
    /// 删除用户手机号
    pub async fn del_mobile<'t>(
        &self,
        user_mobile: &UserMobileModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserMobileModelRef,{
            status:UserMobileStatus::Delete as i8,
            delete_time:time
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, UserMobileModel, _>::new(change)
            .execute_by_pk(user_mobile, &mut db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res= sqlx::query(sql_format!(
                        "UPDATE {} SET mobile_count=mobile_count-1 WHERE id=? and mobile_count-1>=0",
                        UserModel::table_name(),
                    ).as_str())
                    .bind(user_mobile.user_id)
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
                                crate::model::UserIndexCat::Mobile,
                                user_mobile.user_id,
                                &[user_mobile.mobile.to_owned()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&user_mobile.user_id).await;
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
        UserMobileModel,
        UserAccountResult<UserMobileModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_user_id_vec,
        u64,
        UserMobileModel,
        UserAccountResult<Vec<UserMobileModel>>,
        uid,
        "user_id = {uid} and status in ( {status}) order by id desc",
        status = [UserMobileStatus::Init as i8, UserMobileStatus::Valid as i8]
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_user_ids_vec,
        u64,
        UserMobileModel,
        UserAccountResult<HashMap<u64, Vec<UserMobileModel>>>,
        user_id,
        uid,
        "user_id in ({uid}) and status in ({status})  order by id desc",
        status = [UserMobileStatus::Init as i8, UserMobileStatus::Valid as i8]
    );
    pub fn cache(&'_ self) -> UserMobileCache<'_> {
        UserMobileCache { dao: self }
    }
}

pub struct UserMobileCache<'t> {
    pub dao: &'t UserMobile,
}
impl<'t> UserMobileCache<'t> {
    lsys_core::impl_cache_fetch_one!(
        find_by_user_id_vec,
        dao,
        cache,
        u64,
        UserAccountResult<Vec<UserMobileModel>>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_user_ids_vec,
        dao,
        cache,
        u64,
        UserAccountResult<HashMap<u64, Vec<UserMobileModel>>>
    );
}