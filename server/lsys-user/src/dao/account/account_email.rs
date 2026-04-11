use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::AccountResult;

use crate::model::{AccountEmailModel, AccountEmailStatus, AccountModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use lsys_core::valid_code::{CheckCodeData, ValidCode, ValidCodeData, ValidCodeDataRandom};
use lsys_core::valid_param::{ValidEmail, ValidParam, ValidParamCheck, ValidStrlen};
use lsys_core::{db::utils::FetchField, fluent_message, valid_key};

use lsys_core::db::{FieldValue, QueryBuilderExt};
use lsys_core::db::{Insert, TableMeta, Update};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};

use tracing::warn;

use super::AccountError;
use super::AccountIndex;
use super::logger::LogAccountEmail;

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
        let useremal = sqlx::query_as::<_, AccountEmailModel>(&format!(
            "select * from {} where email=? and status in (?,?) order by id desc",
            AccountEmailModel::table_name(),
        ))
        .bind(&email)
        .bind(AccountEmailStatus::Init as i8)
        .bind(AccountEmailStatus::Valid as i8)
        .fetch_one(&self.db)
        .await?;
        Ok(useremal)
    }
    async fn email_param_valid(&self, email: &str) -> AccountResult<()> {
        let email_max = FetchField::new(&self.db)
            .string_max::<AccountEmailModel>(&AccountEmailModel::EMAIL)
            .await
            .len_or(150);

        ValidParam::default()
            .add(
                valid_key!("email"),
                &email,
                &ValidParamCheck::default()
                    .add_rule(ValidEmail::default())
                    .add_rule(ValidStrlen::range(3, email_max)),
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

        let email_res = sqlx::query_as::<_, AccountEmailModel>(&format!(
            "select * from {} where email=? and status in (?,?)",
            AccountEmailModel::table_name(),
        ))
        .bind(email)
        .bind(AccountEmailStatus::Valid as i8)
        .bind(AccountEmailStatus::Init as i8)
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

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<_, AccountEmailModel>::new()
            .set(AccountEmailModel::EMAIL, email_ow)
            .set(AccountEmailModel::ACCOUNT_ID, account.id)
            .set(AccountEmailModel::CHANGE_TIME, time)
            .set(AccountEmailModel::STATUS, _status)
            .execute(&mut *db)
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                use lsys_core::db::Update;
                let res = Update::<_, AccountModel>::new()
                    .set(
                        AccountModel::EMAIL_COUNT,
                        FieldValue::Expr("email_count+1".into()),
                    )
                    .execute(&mut *db, |qb| {
                        qb.push_where().field_eq("id", account.id);
                    })
                    .await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        Err(e.into())
                    }
                    Ok(_) => {
                        if AccountEmailStatus::Valid == status
                            && let Err(ie) = self
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
    pub fn valid_code(&self) -> ValidCode {
        ValidCode::new(self.redis.clone(), "email", true, Some(6))
    }
    /// 获取验证码
    pub async fn valid_code_set<T: ValidCodeData>(
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
    pub fn valid_code_builder(&self) -> ValidCodeDataRandom {
        ValidCodeDataRandom::new(300, 60)
    }
    /// 检测验证码
    pub async fn valid_code_check(
        &self,
        code: &str,
        account_id: u64,
        email: &str,
    ) -> AccountResult<()> {
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
        if res.is_ok()
            && let Err(err) = self.valid_code_clear(email.account_id, &email.email).await
        {
            warn!(
                "email {} valid clear fail:{}",
                &email.email,
                err.to_fluent_message().default_format()
            );
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

        let email_res = sqlx::query_as::<_, AccountEmailModel>(&format!(
            "select * from {} where  email=? and status=? and account_id!=? and id!=?",
            AccountEmailModel::table_name(),
        ))
        .bind(&email.email)
        .bind(AccountEmailStatus::Valid as i8)
        .bind(email.account_id)
        .bind(email.id)
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

        let mut db = self.db.begin().await?;

        let tmp = Update::<_, AccountEmailModel>::new()
            .set(AccountEmailModel::STATUS, AccountEmailStatus::Valid as i8)
            .set(AccountEmailModel::CONFIRM_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", email.id);
            })
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
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<_, AccountEmailModel>::new()
            .set(AccountEmailModel::STATUS, AccountEmailStatus::Delete as i8)
            .set(AccountEmailModel::CHANGE_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", email.id);
            })
            .await;
        match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                use lsys_core::db::Update;
                let res = Update::<_, AccountModel>::new()
                    .set(
                        AccountModel::EMAIL_COUNT,
                        FieldValue::Expr("email_count-1".into()),
                    )
                    .execute(&mut *db, |qb| {
                        qb.push_where().field_eq("id", email.account_id);
                        qb.push_and().field_gte("email_count", 1_i32);
                    })
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

    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountEmailModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountEmailModel>::one(&self.db, |qb| {
            qb.field_eq("id", *id);
            qb.push_and().field_in_copied(
                "status",
                &[
                    AccountEmailStatus::Valid as i8,
                    AccountEmailStatus::Init as i8,
                ],
            );
        })
        .await?)
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountEmailModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountEmailModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("id", ids);
                qb.push_and().field_in_copied(
                    "status",
                    &[
                        AccountEmailStatus::Valid as i8,
                        AccountEmailStatus::Init as i8,
                    ],
                );
            },
            |v| v.id,
        )
        .await?)
    }
    pub async fn find_by_account_id_vec(&self, id: &u64) -> AccountResult<Vec<AccountEmailModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountEmailModel>::vec(&self.db, |qb| {
            qb.field_eq("account_id", *id);
            qb.push_and().field_in_copied(
                "status",
                &[
                    AccountEmailStatus::Init as i8,
                    AccountEmailStatus::Valid as i8,
                ],
            );
            qb.push(" ORDER BY id DESC");
        })
        .await?)
    }
    pub async fn find_by_account_ids_vec(
        &self,
        ids: &[u64],
    ) -> AccountResult<HashMap<u64, Vec<AccountEmailModel>>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountEmailModel>::group(
            &self.db,
            |qb| {
                qb.field_in_copied("account_id", ids);
                qb.push_and().field_in_copied(
                    "status",
                    &[
                        AccountEmailStatus::Init as i8,
                        AccountEmailStatus::Valid as i8,
                    ],
                );
                qb.push(" ORDER BY id DESC");
            },
            |v| v.account_id,
        )
        .await?)
    }
    pub fn cache(&'_ self) -> AccountEmailCache<'_> {
        AccountEmailCache { dao: self }
    }
}

pub struct AccountEmailCache<'t> {
    pub dao: &'t AccountEmail,
}
impl AccountEmailCache<'_> {
    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountEmailModel> {
        self.dao
            .cache
            .get_or_fetch(id, || self.dao.find_by_id(id))
            .await
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountEmailModel>> {
        self.dao
            .cache
            .get_or_fetch_many(ids, |missing| async move {
                self.dao.find_by_ids(&missing).await
            })
            .await
    }
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
