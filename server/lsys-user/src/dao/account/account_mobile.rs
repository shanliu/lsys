use std::collections::HashMap;
use std::sync::Arc;

use crate::dao::AccountResult;

use crate::model::{AccountMobileModel, AccountMobileModelRef, AccountMobileStatus, AccountModel};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{
    fluent_message, now_time, valid_key, RemoteNotify, ValidMobile, ValidParam, ValidParamCheck,
};
use lsys_core::{IntoFluentMessage, RequestEnv};

use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update};
use lsys_core::{model_option_set, sql_format};
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
        let res = sqlx::query_as::<_, AccountMobileModel>(&sql_format!(
            "select * from {} where mobile={} and area_code={}  and status in ({}) order by id desc",
            AccountMobileModel::table_name(),
            mobile,
            area_code,
            &[
                AccountMobileStatus::Init as i8,
                AccountMobileStatus::Valid as i8,
            ]
        ))
        .fetch_one(&self.db)
        .await?;

        Ok(res)
    }
    async fn mobile_param_valid(&self, area_code: &str, mobile: &str) -> AccountResult<()> {
        ValidParam::default()
            .add(
                valid_key!("mobile"),
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
        let mobile_res = sqlx::query_as::<_, AccountMobileModel>(&sql_format!(
            "select * from {} where area_code={} and mobile={} and status in ({})",
            AccountMobileModel::table_name(),
            area_code,
            mobile,
            &[
                AccountMobileStatus::Valid as i8,
                AccountMobileStatus::Init as i8,
            ]
        ))
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
        let idata = model_option_set!(AccountMobileModelRef,{
            mobile:mobile_ow,
            status:_status,
            area_code:area_code_ow,
            account_id:account.id,
            change_time:time,
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let res = Insert::<AccountMobileModel, _>::new(idata)
            .execute(&mut *db)
            .await;
        let aid = match res {
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
            Ok(mr) => {
                let res = sqlx::query(
                    sql_format!(
                        "UPDATE {} SET mobile_count=mobile_count+1 WHERE id=?",
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
                        return Err(e.into());
                    }
                    Ok(_) => {
                        if AccountMobileStatus::Valid == status {
                            if let Err(ie) = self
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
    pub fn valid_code(&self) -> lsys_core::ValidCode {
        lsys_core::ValidCode::new(self.redis.clone(), "mobile", true)
    }
    /// 获取验证码
    pub async fn valid_code_set<T: lsys_core::ValidCodeData>(
        &self,
        valid_code_data: &mut T,
        area_code: &str,
        mobile: &str,
    ) -> lsys_core::ValidCodeResult<(String, usize)> {
        let out = self
            .valid_code()
            .set_code(&format!("{}-{}", area_code, mobile), valid_code_data)
            .await?;
        Ok(out)
    }
    /// 验证码构造器
    pub fn valid_code_builder(&self) -> lsys_core::ValidCodeDataRandom {
        lsys_core::ValidCodeDataRandom::new(120, 30)
    }
    /// 检测验证码
    pub async fn valid_code_check(
        &self,
        code: &str,
        area_code: &str,
        mobile: &str,
    ) -> AccountResult<()> {
        use lsys_core::CheckCodeData;
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
        if res.is_ok() {
            if let Err(err) = self
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

        let mobile_res = sqlx::query_as::<_, AccountMobileModel>(&sql_format!(
            "select * from {} where  area_code={} and mobile={} and status ={} and account_id!={} and id!={}",
            AccountMobileModel::table_name(),
            account_mobile.area_code,
            account_mobile.mobile,
            AccountMobileStatus::Valid,
            account_mobile.account_id,
            account_mobile.id
        ))
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
        let change = lsys_core::model_option_set!(AccountMobileModelRef,{
            status:AccountMobileStatus::Valid as i8,
            confirm_time:time
        });
        let mut db = self.db.begin().await?;

        let tmp = Update::<AccountMobileModel, _>::new(change)
            .execute_by_pk(account_mobile, &mut *db)
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
        let change = lsys_core::model_option_set!(AccountMobileModelRef,{
            status:AccountMobileStatus::Delete as i8,
            change_time:time
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<AccountMobileModel, _>::new(change)
            .execute_by_pk(account_mobile, &mut *db)
            .await;
        let out = match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                let res= sqlx::query(sql_format!(
                        "UPDATE {} SET mobile_count=mobile_count-1 WHERE id=? and mobile_count-1>=0",
                        AccountModel::table_name(),
                    ).as_str())
                    .bind(account_mobile.account_id)
                    .execute(&mut *db).await;
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
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        AccountMobileModel,
        AccountResult<AccountMobileModel>,
        id,
        "id={id} and status in ({status})",
        status = [
            AccountMobileStatus::Valid as i8,
            AccountMobileStatus::Init as i8
        ]
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        AccountMobileModel,
        AccountResult<HashMap<u64, AccountMobileModel>>,
        id,
        ids,
        "id in ({ids}) and status in ({status})",
        status = [
            AccountMobileStatus::Valid as i8,
            AccountMobileStatus::Init as i8
        ]
    );
    lsys_core::impl_dao_fetch_vec_by_one!(
        db,
        find_by_account_id_vec,
        u64,
        AccountMobileModel,
        AccountResult<Vec<AccountMobileModel>>,
        uid,
        "account_id = {uid} and status in ( {status}) order by id desc",
        status = [
            AccountMobileStatus::Init as i8,
            AccountMobileStatus::Valid as i8
        ]
    );
    lsys_core::impl_dao_fetch_vec_by_vec!(
        db,
        find_by_account_ids_vec,
        u64,
        AccountMobileModel,
        AccountResult<HashMap<u64, Vec<AccountMobileModel>>>,
        account_id,
        uid,
        "account_id in ({uid}) and status in ({status})  order by id desc",
        status = [
            AccountMobileStatus::Init as i8,
            AccountMobileStatus::Valid as i8
        ]
    );
    pub fn cache(&'_ self) -> AccountMobileCache<'_> {
        AccountMobileCache { dao: self }
    }
}

pub struct AccountMobileCache<'t> {
    pub dao: &'t AccountMobile,
}
impl AccountMobileCache<'_> {
    lsys_core::impl_cache_fetch_one!(
        find_by_id,
        dao,
        cache,
        u64,
        AccountResult<AccountMobileModel>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_ids,
        dao,
        cache,
        u64,
        AccountResult<HashMap<u64, AccountMobileModel>>
    );
    pub async fn find_by_account_id_vec(
        &self,
        account_id: &u64,
    ) -> AccountResult<Vec<AccountMobileModel>> {
        match self.dao.account_cache.get(account_id).await {
            Some(ids) => Ok(self
                .find_by_ids(&ids)
                .await?
                .into_iter()
                .map(|e| e.1)
                .collect::<Vec<_>>()),
            None => {
                let rows = self.dao.find_by_account_id_vec(account_id).await?;
                for tmp in rows.clone() {
                    self.dao.cache.set(tmp.id, tmp, 0).await;
                }
                let ids = rows.iter().map(|e| e.id).collect::<Vec<_>>();
                self.dao.account_cache.set(*account_id, ids, 0).await;
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
