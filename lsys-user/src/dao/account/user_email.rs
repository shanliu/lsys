use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::account::UserAccountResult;

use crate::model::{UserEmailModel, UserEmailModelRef, UserEmailStatus, UserModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{get_message, now_time, FluentMessage};

use redis::aio::ConnectionManager;

use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::SqlQuote;
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, Select, Update};
use tokio::sync::Mutex;
use tracing::warn;

use super::user_index::UserIndex;
use super::{check_email, UserAccountError};

pub struct UserEmail {
    db: Pool<MySql>,
    redis: Arc<Mutex<ConnectionManager>>,
    fluent: Arc<FluentMessage>,
    index: Arc<UserIndex>,
    pub cache: Arc<LocalCache<u64, Vec<UserEmailModel>>>,
}

impl UserEmail {
    pub fn new(
        db: Pool<MySql>,
        redis: Arc<Mutex<ConnectionManager>>,
        fluent: Arc<FluentMessage>,
        index: Arc<UserIndex>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                redis.clone(),
                LocalCacheConfig::new("user-email"),
            )),
            db,
            redis,
            fluent,
            index,
        }
    }
    /// 根据用户邮箱找到对应的记录
    pub async fn find_by_last_email(&self, email: String) -> UserAccountResult<UserEmailModel> {
        let useremal = Select::type_new::<UserEmailModel>()
            .fetch_one_by_where_call::<UserEmailModel, _, _>(
                "email=? and status in (?,?) order by id desc".to_string(),
                |mut res, _| {
                    res = res.bind(email);
                    res = res.bind(UserEmailStatus::Init as i8);
                    res = res.bind(UserEmailStatus::Valid as i8);
                    res
                },
                &self.db,
            )
            .await?;
        Ok(useremal)
    }
    /// 添加用户邮箱
    pub async fn add_email<'t>(
        &self,
        user: &UserModel,
        email: String,
        status: UserEmailStatus,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        check_email(&self.fluent, email.as_str())?;
        let email_res = Select::type_new::<UserEmailModel>()
            .fetch_one_by_where_call::<UserEmailModel, _, _>(
                " email=? and status in (?,?)".to_string(),
                |mut res, _| {
                    res = res.bind(email.clone());
                    res = res.bind(UserEmailStatus::Valid as i8);
                    res = res.bind(UserEmailStatus::Init as i8);
                    res
                },
                &self.db,
            )
            .await;
        match email_res {
            Ok(email) => {
                if email.user_id == user.id {
                    return Ok(email.id);
                } else {
                    return Err(UserAccountError::System(get_message!(&self.fluent,
                        "user-email-exits","email {$name} bind in other account[{$id}]",
                        ["name"=>email.email,"id"=>email.user_id ]
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
        let idata = model_option_set!(UserEmailModelRef,{
            email:email,
            user_id:user.id,
            add_time:time,
            status:_status,
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<sqlx::MySql, UserEmailModel, _>::new(idata)
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
                        "UPDATE {} SET email_count=email_count+1 WHERE id=?",
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
                        if UserEmailStatus::Valid == status {
                            if let Err(ie) = self
                                .index
                                .add(
                                    crate::model::UserIndexCat::Email,
                                    user.id,
                                    &[email],
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
    impl_account_valid_code_method!("email",{
        user_id:&u64,
        email: &String,
    },{
        &format!("{}-{}", user_id, email)
    },30*60);
    /// 验证验证码并确认用户邮箱
    pub async fn confirm_email_from_code(
        &self,
        email: &UserEmailModel,
        code: &String,
    ) -> UserAccountResult<u64> {
        self.valid_code_check(code, &email.user_id, &email.email)
            .await?;
        let res = self.confirm_email(email).await;
        if res.is_ok() {
            if let Err(err) = self.valid_code_clear(&email.user_id, &email.email).await {
                warn!("email {} valid clear fail:{}", &email.email, err);
            }
        }
        res
    }
    /// 确认用户邮箱
    pub async fn confirm_email(&self, email: &UserEmailModel) -> UserAccountResult<u64> {
        let email_res = Select::type_new::<UserEmailModel>()
            .fetch_one_by_where_call::<UserEmailModel, _, _>(
                " email=? and status = ? and user_id!=? and id!=?".to_string(),
                |mut res, _| {
                    res = res.bind(email.email.clone());
                    res = res.bind(UserEmailStatus::Valid as i8);
                    res = res.bind(email.user_id);
                    res = res.bind(email.id);
                    res
                },
                &self.db,
            )
            .await;
        match email_res {
            Ok(tmp) => {
                return Err(UserAccountError::System(get_message!(&self.fluent,
                    "user-email-exits","comfirn error : email {$name} bind in other account[{$id}]",
                    ["name"=>tmp.email,"id"=>tmp.user_id ]
                )));
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserEmailModelRef,{
            status:UserEmailStatus::Valid as i8,
            confirm_time:time,
        });

        let mut db = self.db.begin().await?;

        let tmp = Update::<sqlx::MySql, UserEmailModel, _>::new(change)
            .execute_by_pk(email, &mut db)
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
                crate::model::UserIndexCat::Email,
                email.user_id,
                &[email.email.to_owned()],
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }

        db.commit().await?;

        self.cache.clear(&email.user_id).await;
        Ok(res.rows_affected())
    }
    /// 删除用户邮箱
    pub async fn del_email<'t>(
        &self,
        email: &UserEmailModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserEmailModelRef,{
            status:UserEmailStatus::Delete as i8,
            delete_time:time,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<sqlx::MySql, UserEmailModel, _>::new(change)
            .execute_by_pk(email, &mut db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res = sqlx::query(
                    sql_format!(
                        "UPDATE {} SET email_count=email_count-1 WHERE id=? and email_count-1>=0",
                        UserModel::table_name(),
                    )
                    .as_str(),
                )
                .bind(email.user_id)
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
                            .del(
                                crate::model::UserIndexCat::Email,
                                email.user_id,
                                &[email.email.to_owned()],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&email.user_id).await;
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
        UserEmailModel,
        UserAccountResult<UserEmailModel>,
        id,
        "id={id}"
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_user_id_vec,
        u64,
        UserEmailModel,
        UserAccountResult<Vec<UserEmailModel>>,
        uid,
        "user_id = {uid} and status in ({status})  order by id desc",
        status = [UserEmailStatus::Init as i8, UserEmailStatus::Valid as i8]
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_user_ids_vec,
        u64,
        UserEmailModel,
        UserAccountResult<HashMap<u64, Vec<UserEmailModel>>>,
        user_id,
        uid,
        "user_id in ({uid}) and status in ({status}) order by id desc",
        status = [UserEmailStatus::Init as i8, UserEmailStatus::Valid as i8]
    );
    pub fn cache(&'_ self) -> UserEmailCache<'_> {
        UserEmailCache { dao: self }
    }
}

pub struct UserEmailCache<'t> {
    pub dao: &'t UserEmail,
}
impl<'t> UserEmailCache<'t> {
    lsys_core::impl_cache_fetch_one!(
        find_by_user_id_vec,
        dao,
        cache,
        u64,
        UserAccountResult<Vec<UserEmailModel>>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_user_ids_vec,
        dao,
        cache,
        u64,
        UserAccountResult<HashMap<u64, Vec<UserEmailModel>>>
    );
}
