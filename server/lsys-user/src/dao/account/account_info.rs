use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    db::utils::FetchField,
    valid_key,
};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{now_time, RequestEnv};
use lsys_core::valid_param::{
    ValidDateTime, ValidIp, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};

use lsys_core::db::{Insert, TableMeta, Update, QueryBuilderExt};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{Acquire, MySql, Pool, Transaction};
use std::{collections::HashMap, sync::Arc};

use crate::model::{AccountInfoModel, AccountModel};

// 用户信息参数结构体
#[derive(Debug, Default)]
pub struct AccountInfoParam<'a> {
    pub gender: Option<i32>,
    pub headimg: Option<&'a str>,
    pub birthday: Option<&'a str>,
    pub reg_ip: Option<&'a str>,
    pub reg_from: Option<&'a str>,
}

use super::{logger::LogAccountInfo, AccountIndex, AccountResult};
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
    async fn info_param_valid(&self, info_param: &AccountInfoParam<'_>) -> AccountResult<()> {
        let fetch_field = FetchField::new(&self.db);
        let headimg_max = fetch_field.string_max::<AccountInfoModel>(&AccountInfoModel::HEADIMG)
            .await
            .len_or(500);
        let reg_from_max = fetch_field.string_max::<AccountInfoModel>(&AccountInfoModel::REG_FROM)
            .await
            .len_or(32);

        let mut param_valid = ValidParam::default();
        if let Some(tmp) = info_param.birthday {
            param_valid.add(
                valid_key!("birthday"),
                &tmp,
                &ValidParamCheck::default().add_rule(ValidDateTime::Date),
            );
        }
        if let Some(tmp) = info_param.gender {
            param_valid.add(
                valid_key!("gender"),
                &tmp,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidNumber::range(0, 3)),
            );
        }
        if let Some(tmp) = info_param.headimg {
            param_valid.add(
                valid_key!("headimg"),
                &tmp,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(0, headimg_max)),
            );
        }
        if let Some(tmp) = info_param.reg_from {
            param_valid.add(
                valid_key!("reg_from"),
                &tmp,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(0, reg_from_max)),
            );
        }
        if let Some(tmp) = info_param.reg_ip {
            param_valid.add(
                valid_key!("reg_ip"),
                &tmp,
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
        info_param: &AccountInfoParam<'_>,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        self.info_param_valid(info_param).await?;
        let time = now_time()?;

        let account_res = sqlx::query_as::<_, AccountInfoModel>(&format!(
            "select * from {} where account_id=?",
            AccountInfoModel::table_name(),
        ))
        .bind(account.id)
        .fetch_one(&self.db)
        .await;

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let tmp = match account_res {
            Err(sqlx::Error::RowNotFound) => {
                if let Some(rf) = info_param.reg_from
                    && !rf.is_empty()
                        && let Err(ie) = self
                            .index
                            .cat_one_add(
                                crate::model::AccountIndexCat::RegFrom,
                                account.id,
                                rf,
                                Some(&mut db),
                            )
                            .await
                        {
                            db.rollback().await?;
                            return Err(ie);
                        }

                let mut insert = Insert::<_,AccountInfoModel>::new()
                    .set(AccountInfoModel::ACCOUNT_ID, account.id)
                    .set(AccountInfoModel::CHANGE_TIME, time);
                if let Some(g) = info_param.gender {
                    insert = insert.set(AccountInfoModel::GENDER, g);
                }
                if let Some(h) = info_param.headimg {
                    insert = insert.set(AccountInfoModel::HEADIMG, h.to_string());
                }
                if let Some(b) = info_param.birthday {
                    insert = insert.set(AccountInfoModel::BIRTHDAY, b.to_string());
                }
                if let Some(ri) = info_param.reg_ip {
                    insert = insert.set(AccountInfoModel::REG_IP, ri.to_string());
                }
                if let Some(rf) = info_param.reg_from {
                    insert = insert.set(AccountInfoModel::REG_FROM, rf.to_string());
                }
                insert.execute(&mut *db).await
            }
            Ok(account_info) => {
                let mut update = Update::<_,AccountInfoModel>::new()
                    .set(AccountInfoModel::ACCOUNT_ID, account.id)
                    .set(AccountInfoModel::CHANGE_TIME, time);
                if let Some(g) = info_param.gender {
                    update = update.set(AccountInfoModel::GENDER, g);
                }
                if let Some(h) = info_param.headimg {
                    update = update.set(AccountInfoModel::HEADIMG, h.to_string());
                }
                if let Some(b) = info_param.birthday {
                    update = update.set(AccountInfoModel::BIRTHDAY, b.to_string());
                }
                if let Some(ri) = info_param.reg_ip {
                    update = update.set(AccountInfoModel::REG_IP, ri.to_string());
                }
                if let Some(rf) = info_param.reg_from {
                    update = update.set(AccountInfoModel::REG_FROM, rf.to_string());
                }
                update
                    .execute(&mut *db, |qb| {
                        qb.push_where().field_eq("id", account_info.id);
                    })
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

        let reg_from_log = info_param
            .reg_from
            .map(|e| e.to_string())
            .unwrap_or_default();
        let reg_ip_log = info_param.reg_ip.map(|e| e.to_string()).unwrap_or_default();
        let birthday_log = info_param
            .birthday
            .map(|e| e.to_string())
            .unwrap_or_default();
        let headimg_log = info_param
            .headimg
            .map(|e| e.to_string())
            .unwrap_or_default();
        let gender_log = info_param.gender.unwrap_or_default();

        self.logger
            .add(
                &LogAccountInfo {
                    gender: gender_log,
                    headimg: &headimg_log,
                    birthday: &birthday_log,
                    reg_ip: &reg_ip_log,
                    reg_from: &reg_from_log,
                },
                Some(account.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    pub async fn find_by_account_id(&self, id: &u64) -> AccountResult<AccountInfoModel> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountInfoModel>::one(
            &self.db,
            |qb| { qb.field_eq("account_id", *id); },
        ).await?)
    }
    pub async fn find_by_account_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountInfoModel>> {
        use lsys_core::db::utils::Fetch;
        Ok(Fetch::<MySql, AccountInfoModel>::map(
            &self.db,
            |qb| {
                qb.field_in_copied("account_id", ids);
            },
            |v| v.account_id,
        ).await?)
    }
    pub fn cache(&'_ self) -> AccountInfoCache<'_> {
        AccountInfoCache { dao: self }
    }
}

pub struct AccountInfoCache<'t> {
    pub dao: &'t AccountInfo,
}
impl AccountInfoCache<'_> {
    pub async fn find_by_account_id(&self, id: &u64) -> AccountResult<AccountInfoModel> {
        self.dao
            .cache
            .get_or_fetch(id, || self.dao.find_by_account_id(id))
            .await
    }
    pub async fn find_by_account_ids(&self, ids: &[u64]) -> AccountResult<HashMap<u64, AccountInfoModel>> {
        self.dao
            .cache
            .get_or_fetch_many(ids, |missing| async move {
                self.dao.find_by_account_ids(&missing).await
            })
            .await
    }
}
