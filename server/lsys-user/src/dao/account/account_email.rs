use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::AccountResult;

use crate::model::{AccountEmailModel, AccountEmailModelRef, AccountEmailStatus, AccountModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{
    fluent_message, now_time, string_clear, valid_key, IntoFluentMessage, RemoteNotify, RequestEnv,
    StringClear, ValidEmail, ValidParam, ValidParamCheck, ValidStrlen, STRING_CLEAR_FORMAT,
};

use lsys_core::db::SqlQuote;
use lsys_core::db::{Insert, ModelTableName, Update};
use lsys_core::{model_option_set, sql_format};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};

use tracing::warn;

use super::logger::LogAccountEmail;
use super::AccountError;
use super::AccountIndex;

pub struct AccountEmail {
    db: Pool<MySql>,
    redis: deadpool_redis::Pool,
    // fluent: Arc<FluentBuild>,
    index: Arc<AccountIndex>,
    pub(crate) cache: Arc<LocalCache<u64, AccountEmailModel>>,
    pub(crate) account_cache: Arc<LocalCache<u64, Vec<u64>>>,
    logger: Arc<ChangeLoggerDao>,
}
impl AccountEmail {
    pub fn new(
        db: Pool<MySql>,
        redis: deadpool_redis::Pool,
        index: Arc<AccountIndex>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
            account_cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
            redis,
            // fluent,
            index,
            logger,
        }
    }
    /// 根据用户邮箱找到对应的记录
    pub async fn find_by_last_email(&self, email: &str) -> AccountResult<AccountEmailModel> {
        let email = string_clear(email, StringClear::Option(STRING_CLEAR_FORMAT), Some(151));
        if email.is_empty() {
            return Err(sqlx::Error::RowNotFound.into());
        }
        let useremal = sqlx::query_as::<_, AccountEmailModel>(&sql_format!(
            "select * from {} where email={} and status in ({}) order by id desc",
            AccountEmailModel::table_name(),
            email,
            &[
                AccountEmailStatus::Init as i8,
                AccountEmailStatus::Valid as i8
            ],
        ))
        .fetch_one(&self.db)
        .await?;
        Ok(useremal)
    }
    async fn email_param_valid(&self, email: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("email"),
                &email,
                &ValidParamCheck::default()
                    .add_rule(ValidEmail::default())
                    .add_rule(ValidStrlen::range(3, 150)),
            )
            .check()?;
        Ok(())
    }

    /// 添加用户邮箱
    pub async fn add_email(
        &self,
        account: &AccountModel,
        email: &str,
        status: AccountEmailStatus,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        self.email_param_valid(email).await?;

        let email_res = sqlx::query_as::<_, AccountEmailModel>(&sql_format!(
            "select * from {} where email={} and status in ({})",
            AccountEmailModel::table_name(),
            email,
            &[
                AccountEmailStatus::Valid as i8,
                AccountEmailStatus::Init as i8
            ]
        ))
        .fetch_one(&self.db)
        .await;

        match email_res {
            Ok(email) => {
                if email.account_id == account.id {
                    return Ok(email.id);
                } else {
                    return Err(AccountError::System(
                        fluent_message!("account-email-exits-other-account",
                            {"email":email.email,"id":email.account_id }
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
        let email_ow = email.to_string();
        let idata = model_option_set!(AccountEmailModelRef,{
            email:email_ow,
            account_id:account.id,
            change_time:time,
            status:_status,
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<AccountEmailModel, _>::new(idata)
            .execute(&mut *db)
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
                        AccountModel::table_name(),
                    )
                    .as_str(),
                )
                .bind(account.id)
                .execute(&mut *db)
                .await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        Err(e.into())
                    }
                    Ok(_) => {
                        if AccountEmailStatus::Valid == status {
                            if let Err(ie) = self
                                .index
                                .add(
                                    crate::model::AccountIndexCat::Email,
                                    account.id,
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
                        self.account_cache.clear(&account.id).await;

                        let aid = mr.last_insert_id();
                        self.logger
                            .add(
                                &LogAccountEmail {
                                    action: "add",
                                    email,
                                    status: status as i8,
                                    account_id: account.id,
                                },
                                Some(aid),
                                Some(op_user_id),
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
}

impl AccountEmail {
    /// 验证码生成
    pub fn valid_code(&self) -> lsys_core::ValidCode {
        lsys_core::ValidCode::new(self.redis.clone(), "email", true, Some(6))
    }
    /// 获取验证码
    pub async fn valid_code_set<T: lsys_core::ValidCodeData>(
        &self,
        valid_code_data: &mut T,
        account_id: u64,
        email: &str,
    ) -> AccountResult<(String, usize)> {
        self.email_param_valid(email).await?;
        let out = self
            .valid_code()
            .set_code(&format!("{}-{}", account_id, email), valid_code_data)
            .await?;
        Ok(out)
    }
    /// 验证码构造器
    pub fn valid_code_builder(&self) -> lsys_core::ValidCodeDataRandom {
        lsys_core::ValidCodeDataRandom::new(300, 60)
    }
    /// 检测验证码
    pub async fn valid_code_check(
        &self,
        code: &str,
        account_id: u64,
        email: &str,
    ) -> AccountResult<()> {
        use lsys_core::CheckCodeData;

        self.valid_code()
            .check_code(&CheckCodeData::new(
                &format!("{}-{}", account_id, email),
                code,
            ))
            .await?;
        Ok(())
    }
    pub async fn valid_code_clear(&self, account_id: u64, email: &str) -> AccountResult<()> {
        let mut builder = self.valid_code_builder();
        self.valid_code()
            .destroy_code(&format!("{}-{}", account_id, email), &mut builder)
            .await?;
        Ok(())
    }
}
impl AccountEmail {
    /// 验证验证码并确认用户邮箱
    pub async fn confirm_email_from_code(
        &self,
        email: &AccountEmailModel,
        code: &str,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        if AccountEmailStatus::Delete.eq(email.status) {
            return Err(AccountError::System(fluent_message!("email-bad-status",
                {"email":&email.email}
            )));
        }
        self.valid_code_check(code, email.account_id, &email.email)
            .await?;

        let res = self.confirm_email(email, op_user_id, env_data).await;
        if res.is_ok() {
            if let Err(err) = self.valid_code_clear(email.account_id, &email.email).await {
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
        email: &AccountEmailModel,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        if AccountEmailStatus::Valid.eq(email.status) {
            return Ok(0);
        }

        let email_res = sqlx::query_as::<_, AccountEmailModel>(&sql_format!(
            "select * from {} where  email={} and status = {} and account_id!={} and id!={}",
            AccountEmailModel::table_name(),
            email.email,
            AccountEmailStatus::Valid,
            email.account_id,
            email.id
        ))
        .fetch_one(&self.db)
        .await;

        match email_res {
            Ok(tmp) => {
                return Err(AccountError::System(
                    fluent_message!("account-email-exits-other-account",
                        {"email":tmp.email,"id":tmp.account_id }
                    ),
                )); //"comfirn error : email {$name} bind in other account[{$id}]",
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let time = now_time()?;
        let change = lsys_core::model_option_set!(AccountEmailModelRef,{
            status:AccountEmailStatus::Valid as i8,
            confirm_time:time,
        });

        let mut db = self.db.begin().await?;

        let tmp = Update::<AccountEmailModel, _>::new(change)
            .execute_by_pk(email, &mut *db)
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
                crate::model::AccountIndexCat::Email,
                email.account_id,
                &[&email.email],
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }

        db.commit().await?;
        self.cache.clear(&email.id).await;
        self.account_cache.clear(&email.account_id).await;

        self.logger
            .add(
                &LogAccountEmail {
                    action: "confirm",
                    account_id: email.account_id,
                    email: &email.email,
                    status: AccountEmailStatus::Valid as i8,
                },
                Some(email.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.rows_affected())
    }
    /// 删除用户邮箱
    pub async fn del_email(
        &self,
        email: &AccountEmailModel,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        if AccountEmailStatus::Delete.eq(email.status) {
            return Ok(0_u64);
        }
        let time = now_time()?;
        let change = lsys_core::model_option_set!(AccountEmailModelRef,{
            status:AccountEmailStatus::Delete as i8,
            change_time:time,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<AccountEmailModel, _>::new(change)
            .execute_by_pk(email, &mut *db)
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
                        AccountModel::table_name(),
                    )
                    .as_str(),
                )
                .bind(email.account_id)
                .execute(&mut *db)
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
                                crate::model::AccountIndexCat::Email,
                                email.account_id,
                                &[&email.email],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.cache.clear(&email.id).await;
                        self.account_cache.clear(&email.account_id).await;

                        self.logger
                            .add(
                                &LogAccountEmail {
                                    action: "del",
                                    account_id: email.account_id,
                                    email: &email.email,
                                    status: AccountEmailStatus::Valid as i8,
                                },
                                Some(email.id),
                                Some(op_user_id),
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
        AccountEmailModel,
        AccountResult<AccountEmailModel>,
        id,
        "id={id} and status in ({status})",
        status = [
            AccountEmailStatus::Valid as i8,
            AccountEmailStatus::Init as i8
        ]
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        AccountEmailModel,
        AccountResult<HashMap<u64, AccountEmailModel>>,
        id,
        ids,
        "id in ({ids}) and status in ({status})",
        status = [
            AccountEmailStatus::Valid as i8,
            AccountEmailStatus::Init as i8
        ]
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_account_id_vec,
        u64,
        AccountEmailModel,
        AccountResult<Vec<AccountEmailModel>>,
        uid,
        "account_id = {uid} and status in ({status})  order by id desc",
        status = [
            AccountEmailStatus::Init as i8,
            AccountEmailStatus::Valid as i8
        ]
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_account_ids_vec,
        u64,
        AccountEmailModel,
        AccountResult<HashMap<u64, Vec<AccountEmailModel>>>,
        account_id,
        uid,
        "account_id in ({uid}) and status in ({status}) order by id desc",
        status = [
            AccountEmailStatus::Init as i8,
            AccountEmailStatus::Valid as i8
        ]
    );
    pub fn cache(&'_ self) -> AccountEmailCache<'_> {
        AccountEmailCache { dao: self }
    }
}

pub struct AccountEmailCache<'t> {
    pub dao: &'t AccountEmail,
}
impl AccountEmailCache<'_> {
    lsys_core::impl_cache_fetch_one!(
        find_by_id,
        dao,
        cache,
        u64,
        AccountResult<AccountEmailModel>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_ids,
        dao,
        cache,
        u64,
        AccountResult<HashMap<u64, AccountEmailModel>>
    );
    pub async fn find_by_account_id_vec(
        &self,
        account_id: u64,
    ) -> AccountResult<Vec<AccountEmailModel>> {
        match self.dao.account_cache.get(&account_id).await {
            Some(ids) => Ok(self
                .find_by_ids(&ids)
                .await?
                .into_iter()
                .map(|e| e.1)
                .collect::<Vec<_>>()),
            None => {
                let rows = self.dao.find_by_account_id_vec(&account_id).await?;
                for tmp in rows.clone() {
                    self.dao.cache.set(tmp.id, tmp, 0).await;
                }
                let ids = rows.iter().map(|e| e.id).collect::<Vec<_>>();
                self.dao.account_cache.set(account_id, ids, 0).await;
                Ok(rows)
            }
        }
    }
    pub async fn find_by_account_ids_vec(
        &self,
        account_ids: &[u64],
    ) -> AccountResult<HashMap<u64, Vec<AccountEmailModel>>> {
        let mut get = vec![];
        let mut hash = std::collections::HashMap::with_capacity(account_ids.len());
        for account_id in account_ids {
            match self.dao.account_cache.get(account_id).await {
                Some(ids) => {
                    let data = self
                        .find_by_ids(&ids)
                        .await?
                        .into_iter()
                        .map(|e| e.1)
                        .collect::<Vec<AccountEmailModel>>();
                    hash.entry(*account_id).or_insert(data);
                }
                None => {
                    get.push(*account_id);
                }
            }
        }
        if !get.is_empty() {
            match self.dao.find_by_account_ids_vec(&get).await {
                Ok(datas) => {
                    for (pk, rows) in datas.into_iter() {
                        hash.entry(pk).or_default().extend(rows.clone());
                        for tmp in rows.clone() {
                            self.dao.cache.set(tmp.id, tmp, 0).await;
                        }
                        let ids = rows.iter().map(|e| e.id).collect::<Vec<_>>();
                        self.dao.account_cache.set(pk, ids, 0).await;
                    }
                }
                Err(err) => return Err(err),
            }
        }
        Ok(hash)
    }
}
