use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::account::UserAccountResult;

use crate::model::{UserEmailModel, UserEmailModelRef, UserEmailStatus, UserModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{fluent_message, now_time, IntoFluentMessage, RemoteNotify, RequestEnv};

use lsys_logger::dao::ChangeLogger;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::SqlQuote;
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, Select, Update};

use tracing::warn;

use super::logger::LogUserEmail;
use super::user_index::UserIndex;
use super::{check_email, UserAccountError};

pub struct UserEmail {
    db: Pool<MySql>,
    redis: deadpool_redis::Pool,
    // fluent: Arc<FluentBuild>,
    index: Arc<UserIndex>,
    pub(crate) cache: Arc<LocalCache<u64, Vec<UserEmailModel>>>,
    logger: Arc<ChangeLogger>,
}

impl UserEmail {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        // fluent: Arc<FluentBuild>,
        remote_notify: Arc<RemoteNotify>,
        index: Arc<UserIndex>,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(
                remote_notify,
                LocalCacheConfig::new("user-email"),
            )),
            db,
            redis,
            // fluent,
            index,
            logger,
        }
    }
    /// 根据用户邮箱找到对应的记录
    pub async fn find_by_last_email(&self, email: String) -> UserAccountResult<UserEmailModel> {
        let useremal = Select::type_new::<UserEmailModel>()
            .fetch_one_by_where::<UserEmailModel, _>(
                &sqlx_model::WhereOption::Where(sql_format!(
                    "email={} and status in ({}) order by id desc",
                    email,
                    &[UserEmailStatus::Init as i8, UserEmailStatus::Valid as i8],
                )),
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
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<u64> {
        check_email(email.as_str())?;
        let email_res = Select::type_new::<UserEmailModel>()
            .fetch_one_by_where::<UserEmailModel, _>(
                &sqlx_model::WhereOption::Where(sql_format!(
                    " email={} and status in ({})",
                    email,
                    &[UserEmailStatus::Valid as i8, UserEmailStatus::Init as i8]
                )),
                &self.db,
            )
            .await;
        match email_res {
            Ok(email) => {
                if email.user_id == user.id {
                    return Ok(email.id);
                } else {
                    return Err(UserAccountError::System(
                        fluent_message!("user-email-exits-other-account",
                            {"email":email.email,"id":email.user_id }
                        ),
                    )); //"email {$name} bind in other account[{$id}]",
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
            change_time:time,
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
                                    &[email.clone()],
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

                        let aid = mr.last_insert_id();

                        self.logger
                            .add(
                                &LogUserEmail {
                                    action: "add",
                                    email,
                                    status: status as i8,
                                },
                                &Some(aid),
                                &Some(user.id),
                                &Some(user.id),
                                None,
                                env_data,
                            )
                            .await;

                        Ok(aid)
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
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<u64> {
        self.valid_code_check(code, &email.user_id, &email.email)
            .await?;
        let res = self.confirm_email(email, env_data).await;
        if res.is_ok() {
            if let Err(err) = self.valid_code_clear(&email.user_id, &email.email).await {
                warn!(
                    "email {} valid clear fail:{}",
                    &email.email,
                    err.to_fluent_message().default_format()
                );
            }
        }
        res
    }
    /// 确认用户邮箱
    pub async fn confirm_email(
        &self,
        email: &UserEmailModel,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<u64> {
        let email_res = Select::type_new::<UserEmailModel>()
            .fetch_one_by_where::<UserEmailModel, _>(
                &sqlx_model::WhereOption::Where(sql_format!(
                    " email={} and status = {} and user_id!={} and id!={}",
                    email.email,
                    UserEmailStatus::Valid,
                    email.user_id,
                    email.id
                )),
                &self.db,
            )
            .await;
        match email_res {
            Ok(tmp) => {
                return Err(UserAccountError::System(
                    fluent_message!("user-email-exits-other-account",
                        {"email":tmp.email,"id":tmp.user_id }
                    ),
                )); //"comfirn error : email {$name} bind in other account[{$id}]",
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

        self.logger
            .add(
                &LogUserEmail {
                    action: "confirm",
                    email: email.email.to_owned(),
                    status: UserEmailStatus::Valid as i8,
                },
                &Some(email.id),
                &Some(email.user_id),
                &Some(email.user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.rows_affected())
    }
    /// 删除用户邮箱
    pub async fn del_email<'t>(
        &self,
        email: &UserEmailModel,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<u64> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(UserEmailModelRef,{
            status:UserEmailStatus::Delete as i8,
            change_time:time,
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

                        self.logger
                            .add(
                                &LogUserEmail {
                                    action: "del",
                                    email: email.email.to_owned(),
                                    status: UserEmailStatus::Valid as i8,
                                },
                                &Some(email.id),
                                &Some(email.user_id),
                                &Some(email.user_id),
                                None,
                                env_data,
                            )
                            .await;

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
