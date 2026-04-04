use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::AccountResult;

use crate::model::{AccountMobileModel, AccountMobileStatus, AccountModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{
    now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT,
};
use lsys_core::valid_code::{CheckCodeData, ValidCode, ValidCodeData, ValidCodeDataRandom};
use lsys_core::valid_param::{ValidMobile, ValidParam, ValidParamCheck};
use lsys_core::{fluent_message, valid_key};

use lsys_core::db::{FieldValue, Insert, QueryBuilderExt, TableMeta, Update};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};

use tracing::log::warn;

use super::logger::LogAccountMobile;
use super::AccountError;
use super::AccountIndex;

pub struct AccountMobile {
    db: Pool<MySql>,
    redis: deadpool_redis::Pool,
    index: Arc<AccountIndex>,
    pub(crate) cache: Arc<LocalCache<u64, AccountMobileModel>>,
    pub(crate) account_cache: Arc<LocalCache<u64, Vec<u64>>>,
    logger: Arc<ChangeLoggerDao>,
}

impl AccountMobile {
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
            index,
            logger,
        }
    }

    /// 通过手机号查找用户手机号记录
    pub async fn find_by_last_mobile(
        &self,
        area_code: &str,
        mobile: &str,
    ) -> AccountResult<AccountMobileModel> {
        let area_code = string_clear(area_code, StringClear::Option(STRING_CLEAR_FORMAT), Some(5));
        if area_code.is_empty() {
            return Err(sqlx::Error::RowNotFound.into());
        }
        let mobile = string_clear(mobile, StringClear::Option(STRING_CLEAR_FORMAT), Some(14));
        if mobile.is_empty() {
            return Err(sqlx::Error::RowNotFound.into());
        }
        let res = sqlx::query_as::<_, AccountMobileModel>(&format!(
            "select * from {} where mobile=? and area_code=?  and status in (?,?) order by id desc",
            AccountMobileModel::table_name(),
        ))
        .bind(&mobile)
        .bind(&area_code)
        .bind(AccountMobileStatus::Init as i8)
        .bind(AccountMobileStatus::Valid as i8)
        .fetch_one(&self.db)
        .await?;

        Ok(res)
    }
    async fn mobile_param_valid(&self, area_code: &str, mobile: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("user_mobile"),
                &format!("{}{}", area_code, mobile),
                &ValidParamCheck::default().add_rule(ValidMobile::default()),
            )
            .check()?;

        Ok(())
    }
    /// 添加手机号
    #[allow(clippy::too_many_arguments)]
    pub async fn add_mobile(
        &self,
        account: &AccountModel,
        area_code: &str,
        mobile: &str,
        mut status: AccountMobileStatus,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        self.mobile_param_valid(area_code, mobile).await?;
        if status == AccountMobileStatus::Delete {
            status = AccountMobileStatus::Init;
        }
        let mobile_res = sqlx::query_as::<_, AccountMobileModel>(&format!(
            "select * from {} where area_code=? and mobile=? and status in (?,?)",
            AccountMobileModel::table_name(),
        ))
        .bind(area_code)
        .bind(mobile)
        .bind(AccountMobileStatus::Valid as i8)
        .bind(AccountMobileStatus::Init as i8)
        .fetch_one(&self.db)
        .await;

        match mobile_res {
            Ok(mobile) => {
                if mobile.account_id == account.id {
                    return Ok(mobile.id);
                } else {
                    return Err(AccountError::System(
                        fluent_message!("account-mobile-exits",
                            {"mobile":mobile.mobile,"id":mobile.account_id }//"mobile {$name} bind on other account[{$id}]",
                        ),
                    ));
                }
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }

        let time = now_time()?;
        let _status = status as i8;
        let area_code_ow = area_code.to_string();
        let mobile_ow = mobile.to_string();

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<_,AccountMobileModel>::new()
            .set(AccountMobileModel::MOBILE, mobile_ow)
            .set(AccountMobileModel::STATUS, _status)
            .set(AccountMobileModel::AREA_CODE, area_code_ow)
            .set(AccountMobileModel::ACCOUNT_ID, account.id)
            .set(AccountMobileModel::CHANGE_TIME, time)
            .execute(&mut *db)
            .await;
        let aid = match res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => {
                use lsys_core::db::Update;
                let res = Update::<_, AccountModel>::new()
                    .set(AccountModel::MOBILE_COUNT, FieldValue::Expr("mobile_count+1".into()))
                    .execute(&mut *db, |qb| {
                        qb.push_where().field_eq("id", account.id);
                    })
                    .await;
                match res {
                    Err(e) => {
                        db.rollback().await?;
                        return Err(e.into());
                    }
                    Ok(_) => {
                        if AccountMobileStatus::Valid == status
                            && let Err(ie) = self
                                .index
                                .add(
                                    crate::model::AccountIndexCat::Mobile,
                                    account.id,
                                    &[mobile],
                                    Some(&mut db),
                                )
                                .await
                            {
                                db.rollback().await?;
                                return Err(ie);
                            }

                        db.commit().await?;
                        self.account_cache.clear(&account.id).await;
                        mr.last_insert_id()
                    }
                }
            }
        };

        self.logger
            .add(
                &LogAccountMobile {
                    action: "add",
                    area_code,
                    mobile,
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

impl AccountMobile {
    /// 验证码生成
    pub fn valid_code(&self) -> ValidCode {
        ValidCode::new(self.redis.clone(), "mobile", true, Some(6))
    }
    /// 获取验证码
    pub async fn valid_code_set<T: ValidCodeData>(
        &self,
        valid_code_data: &mut T,
        area_code: &str,
        mobile: &str,
    ) -> AccountResult<(String, usize)> {
        self.mobile_param_valid(area_code, mobile).await?;
        let out = self
            .valid_code()
            .set_code(&format!("{}-{}", area_code, mobile), valid_code_data)
            .await?;
        Ok(out)
    }
    /// 验证码构造器
    pub fn valid_code_builder(&self) -> ValidCodeDataRandom {
        ValidCodeDataRandom::new(120, 30)
    }
    /// 检测验证码
    pub async fn valid_code_check(
        &self,
        code: &str,
        area_code: &str,
        mobile: &str,
    ) -> AccountResult<()> {
        self.valid_code()
            .check_code(&CheckCodeData::new(
                &format!("{}-{}", area_code, mobile),
                code,
            ))
            .await?;
        Ok(())
    }
    pub async fn valid_code_clear(&self, area_code: &str, mobile: &str) -> AccountResult<()> {
        let mut builder = self.valid_code_builder();
        self.valid_code()
            .destroy_code(&format!("{}-{}", area_code, mobile), &mut builder)
            .await?;
        Ok(())
    }
}
impl AccountMobile {
    /// 验证code并确认手机号
    pub async fn confirm_mobile_from_code(
        &self,
        account_mobile: &AccountMobileModel,
        code: &str,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        if AccountMobileStatus::Delete.eq(account_mobile.status) {
            return Err(AccountError::System(fluent_message!("mobile-bad-status",
                {"mobile":&account_mobile.mobile}
            )));
        }
        self.valid_code_check(code, &account_mobile.area_code, &account_mobile.mobile)
            .await?;
        let res = self
            .confirm_mobile(account_mobile, op_user_id, env_data)
            .await;
        if res.is_ok()
            && let Err(err) = self
                .valid_code_clear(&account_mobile.area_code, &account_mobile.mobile)
                .await
            {
                warn!(
                    "mobile {}-{} valid clear fail:{}",
                    &account_mobile.area_code,
                    &account_mobile.mobile,
                    err.to_fluent_message().default_format()
                );
            }
        res
    }
    //确认手机号
    pub async fn confirm_mobile(
        &self,
        account_mobile: &AccountMobileModel,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        if AccountMobileStatus::Valid.eq(account_mobile.status) {
            return Ok(0);
        }

        let mobile_res = sqlx::query_as::<_, AccountMobileModel>(&format!(
            "select * from {} where  area_code=? and mobile=? and status=? and account_id!=? and id!=?",
            AccountMobileModel::table_name(),
        ))
        .bind(&account_mobile.area_code)
        .bind(&account_mobile.mobile)
        .bind(AccountMobileStatus::Valid as i8)
        .bind(account_mobile.account_id)
        .bind(account_mobile.id)
        .fetch_one(&self.db)
        .await;

        match mobile_res {
            Ok(mobile) => {
                return Err(AccountError::System(
                    fluent_message!("account-mobile-exits",
                        {"mobile":mobile.mobile,"id":mobile.account_id }//"confirm error : mobile {$name} bind on other account[{$id}]",
                    ),
                ));
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => {
                return Err(err.into());
            }
        }
        let time = now_time()?;

        let mut db = self.db.begin().await?;

        let tmp = Update::<_,AccountMobileModel>::new()
            .set(AccountMobileModel::STATUS, AccountMobileStatus::Valid as i8)
            .set(AccountMobileModel::CONFIRM_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", account_mobile.id);
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
                crate::model::AccountIndexCat::Mobile,
                account_mobile.account_id,
                &[&account_mobile.mobile],
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;
        self.account_cache.clear(&account_mobile.account_id).await;
        self.cache.clear(&account_mobile.id).await;

        self.logger
            .add(
                &LogAccountMobile {
                    action: "confirm",
                    area_code: &account_mobile.area_code,
                    mobile: &account_mobile.mobile,
                    status: AccountMobileStatus::Valid as i8,
                    account_id: account_mobile.account_id,
                },
                Some(account_mobile.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.rows_affected())
    }
    /// 删除用户手机号
    pub async fn del_mobile(
        &self,
        account_mobile: &AccountMobileModel,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        if AccountMobileStatus::Delete.eq(account_mobile.status) {
            return Ok(0_u64);
        }
        let time = now_time()?;
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<_,AccountMobileModel>::new()
            .set(AccountMobileModel::STATUS, AccountMobileStatus::Delete as i8)
            .set(AccountMobileModel::CHANGE_TIME, time)
            .execute(&mut *db, |qb| {
                qb.push_where().field_eq("id", account_mobile.id);
            })
            .await;
        let out = match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                use lsys_core::db::Update;
                let res= Update::<_, AccountModel>::new()
                    .set(AccountModel::MOBILE_COUNT, FieldValue::Expr("mobile_count-1".into()))
                    .execute(&mut *db, |qb| {
                        qb.push_where().field_eq("id", account_mobile.account_id);
                        qb.push_and().field_gte("mobile_count", 1_i32);
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
                                crate::model::AccountIndexCat::Mobile,
                                account_mobile.account_id,
                                &[&account_mobile.mobile],
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                        db.commit().await?;
                        self.account_cache.clear(&account_mobile.account_id).await;
                        self.cache.clear(&account_mobile.id).await;
                        Ok(mr.rows_affected())
                    }
                }
            }
        };

        self.logger
            .add(
                &LogAccountMobile {
                    action: "del",
                    area_code: &account_mobile.area_code,
                    mobile: &account_mobile.mobile,
                    status: AccountMobileStatus::Valid as i8,
                    account_id: account_mobile.account_id,
                },
                Some(account_mobile.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;
        out
    }
    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountMobileModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountMobileModel>::one(
            &self.db,
            |qb| {
                qb.field_eq("id", *id);
                qb.push_and().field_in_copied("status", &[AccountMobileStatus::Valid as i8, AccountMobileStatus::Init as i8]);
            },
        ).await?)
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountMobileModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountMobileModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("id", ids);
                qb.push_and().field_in_copied("status", &[AccountMobileStatus::Valid as i8, AccountMobileStatus::Init as i8]);
            },
            |v| v.id,
        ).await?)
    }
    pub async fn find_by_account_id_vec(&self, id: &u64) -> AccountResult<Vec<AccountMobileModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountMobileModel>::vec(
            &self.db,
            |qb| {
                qb.field_eq("account_id", *id);
                qb.push_and().field_in_copied("status", &[AccountMobileStatus::Init as i8, AccountMobileStatus::Valid as i8]);
                qb.push(" ORDER BY id DESC");
            },
        ).await?)
    }
    pub async fn find_by_account_ids_vec(&self, ids: &[u64]) -> AccountResult<HashMap<u64, Vec<AccountMobileModel>>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountMobileModel>::group(
            &self.db,
            |qb| {
                qb.field_in_copied("account_id", ids);
                qb.push_and().field_in_copied("status", &[AccountMobileStatus::Init as i8, AccountMobileStatus::Valid as i8]);
                qb.push(" ORDER BY id DESC");
            },
            |v| v.account_id,
        ).await?)
    }
    pub fn cache(&'_ self) -> AccountMobileCache<'_> {
        AccountMobileCache { dao: self }
    }
}

pub struct AccountMobileCache<'t> {
    pub dao: &'t AccountMobile,
}
impl AccountMobileCache<'_> {
    pub async fn find_by_id(&self, id: &u64) -> AccountResult<AccountMobileModel> {
        self.dao
            .cache
            .get_or_fetch(id, || self.dao.find_by_id(id))
            .await
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountMobileModel>> {
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
    ) -> AccountResult<Vec<AccountMobileModel>> {
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
    ) -> AccountResult<HashMap<u64, Vec<AccountMobileModel>>> {
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
                        .collect::<Vec<AccountMobileModel>>();
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
