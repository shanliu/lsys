use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    now_time, valid_key, RemoteNotify, RequestEnv, ValidDateTime, ValidIp, ValidNumber, ValidParam,
    ValidParamCheck, ValidPattern, ValidStrlen,
};

use lsys_core::db::{Insert, ModelTableName, Update};
use lsys_core::sql_format;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};
use std::{collections::HashMap, sync::Arc};

use crate::model::{AccountInfoModel, AccountInfoModelRef, AccountModel};

use super::{logger::LogAccountInfo, AccountIndex, AccountResult};
use lsys_core::db::SqlQuote;
pub struct AccountInfo {
    db: Pool<MySql>,

    index: Arc<AccountIndex>,
    pub(crate) cache: Arc<LocalCache<u64, AccountInfoModel>>,
    logger: Arc<ChangeLoggerDao>,
}

impl AccountInfo {
    pub fn new(
        db: Pool<MySql>,

        index: Arc<AccountIndex>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
            logger,
            index,
        }
    }
    async fn info_param_valid(&self, info: &AccountInfoModelRef<'_>) -> AccountResult<()> {
        let mut param_valid = ValidParam::default();
        if let Some(tmp) = info.birthday {
            param_valid.add(
                valid_key!("birthday"),
                tmp,
                &ValidParamCheck::default().add_rule(ValidDateTime::Date),
            );
        }
        if let Some(tmp) = info.gender {
            param_valid.add(
                valid_key!("gender"),
                tmp,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidNumber::range(1, 3)),
            );
        }
        if let Some(tmp) = info.headimg {
            param_valid.add(
                valid_key!("headimg"),
                tmp,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(3, 512)),
            );
        }
        if let Some(tmp) = info.reg_from {
            param_valid.add(
                valid_key!("reg_from"),
                tmp,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(0, 32)),
            );
        }
        if let Some(tmp) = info.reg_ip {
            param_valid.add(
                valid_key!("reg_ip"),
                tmp,
                &ValidParamCheck::default().add_rule(ValidIp::default()),
            );
        }
        param_valid.check()?;
        Ok(())
    }
    /// 设置用户信息
    pub async fn set_info(
        &self,
        account: &AccountModel,
        info: &AccountInfoModelRef<'_>,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        self.info_param_valid(info).await?;
        let time = now_time()?;
        let set_info = AccountInfoModelRef {
            id: None,
            account_id: Some(&account.id),
            gender: info.gender,
            headimg: info.headimg,
            birthday: info.birthday,
            reg_ip: info.reg_ip,
            reg_from: info.reg_from,
            change_time: Some(&time),
        };

        let account_res = sqlx::query_as::<_, AccountInfoModel>(&sql_format!(
            "select * from {} where account_id={}",
            AccountInfoModel::table_name(),
            account.id
        ))
        .fetch_one(&self.db)
        .await;

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let tmp = match account_res {
            Err(sqlx::Error::RowNotFound) => {
                if let Some(rf) = info.reg_from {
                    if !rf.is_empty() {
                        if let Err(ie) = self
                            .index
                            .cat_one_add(
                                crate::model::AccountIndexCat::RegFrom,
                                account.id,
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

                Insert::<AccountInfoModel, _>::new(set_info)
                    .execute(&mut *db)
                    .await
            }
            Ok(account_info) => {
                Update::<AccountInfoModel, _>::new(set_info)
                    .execute_by_pk(&account_info, &mut *db)
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
        self.cache.clear(&account.id).await;

        let reg_from = info.reg_from.map(|e| e.to_string()).unwrap_or_default();
        let reg_ip = info.reg_ip.map(|e| e.to_string()).unwrap_or_default();
        let birthday = info.birthday.map(|e| e.to_string()).unwrap_or_default();
        let headimg = info.headimg.map(|e| e.to_string()).unwrap_or_default();
        let gender = info.gender.map(|e| e.to_owned()).unwrap_or_default();

        self.logger
            .add(
                &LogAccountInfo {
                    gender,
                    headimg: &headimg,
                    birthday: &birthday,
                    reg_ip: &reg_ip,
                    reg_from: &reg_from,
                },
                Some(account.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_account_id,
        u64,
        AccountInfoModel,
        AccountResult<AccountInfoModel>,
        id,
        "account_id = {id}"
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_account_ids,
        u64,
        AccountInfoModel,
        AccountResult<HashMap<u64, AccountInfoModel>>,
        account_id,
        ids,
        "account_id in ({ids})"
    );
    pub fn cache(&'_ self) -> AccountInfoCache<'_> {
        AccountInfoCache { dao: self }
    }
}

pub struct AccountInfoCache<'t> {
    pub dao: &'t AccountInfo,
}
impl AccountInfoCache<'_> {
    lsys_core::impl_cache_fetch_one!(
        find_by_account_id,
        dao,
        cache,
        u64,
        AccountResult<AccountInfoModel>
    );
    lsys_core::impl_cache_fetch_vec!(
        find_by_account_ids,
        dao,
        cache,
        u64,
        AccountResult<HashMap<u64, AccountInfoModel>>
    );
}
