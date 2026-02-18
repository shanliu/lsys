use std::collections::HashMap;

use std::sync::Arc;

use crate::dao::AccountResult;
use crate::model::{AccountIndexCat, AccountModel, AccountStatus};

use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{CursorPageData, CursorPageParam};
use lsys_core::{
    db::query_string_field_max, fluent_message, now_time, valid_key, RemoteNotify, RequestEnv,
    ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};
use lsys_logger::dao::ChangeLoggerDao;

use super::logger::LogAccount;
use super::AccountError;
use super::{AccountIndex, AccountItem};
use lsys_core::db::{Insert, SqlSuffix, Update};
use lsys_core::db::{SqlQuote, TableMeta};
use lsys_core::sql_format;
use sqlx::{Acquire, MySql, Pool, Transaction};
pub struct Account {
    db: Pool<MySql>,
    index: Arc<AccountIndex>,
    pub(crate) cache: Arc<LocalCache<u64, AccountModel>>,
    logger: Arc<ChangeLoggerDao>,
}

// find_by_id_impl!(Account,AccountModel,cache,id,"");

impl Account {
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
            index,
            logger,
        }
    }
    async fn nickname_param_valid(&self, nickname: &str) -> AccountResult<()> {
        let nickname_max =
            query_string_field_max::<AccountModel>(&self.db, &AccountModel::NICKNAME)
                .await
                .len_or(32);

        ValidParam::default()
            .add(
                valid_key!("nickname"),
                &nickname,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, nickname_max)),
            )
            .check()?;

        Ok(())
    }
    /// 添加用户
    pub async fn add(
        &self,
        nickname: &str,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<AccountModel> {
        self.nickname_param_valid(nickname).await?;
        let time = now_time()?;
        let u_status = AccountStatus::Init as i8;
        let nickname_ow = nickname.to_string();

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Insert::<AccountModel>::new()
            .set(AccountModel::NICKNAME, nickname_ow)
            .set(AccountModel::ADD_TIME, time)
            .set(AccountModel::CHANGE_TIME, time)
            .set(AccountModel::USE_NAME, 0i8)
            .set(AccountModel::STATUS, u_status)
            .execute(&mut *db)
            .await;
        let res = match tmp {
            Ok(e) => e,
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
        };
        let account_id = res.last_insert_id();

        let tmp = sqlx::query_as::<_, AccountModel>(&sql_format!(
            "select * from {} where id={} ",
            AccountModel::table_name(),
            account_id
        ))
        .fetch_one(&mut *db)
        .await;

        let account = match tmp {
            Ok(e) => e,
            Err(ie) => {
                db.rollback().await?;
                return Err(ie.into());
            }
        };
        if let Err(ie) = self
            .index
            .cat_one_add(
                crate::model::AccountIndexCat::AccountStatus,
                account.id,
                &(AccountStatus::Init as i8).to_string(),
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;

        self.logger
            .add(
                &LogAccount {
                    action: "add",
                    nickname,
                    status: u_status,
                },
                Some(account.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(account)
    }
    //激活用户
    pub async fn enable(
        &self,
        account: &AccountModel,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        if AccountStatus::Delete.eq(account.status) {
            return Err(AccountError::System(fluent_message!("account-is-delete",{
                "account":&account.nickname
            })));
        }
        if AccountStatus::Enable.eq(account.status) {
            return Ok(());
        }
        let time = now_time().unwrap_or_default();
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::<AccountModel>::new()
            .set(AccountModel::CHANGE_TIME, time)
            .set(AccountModel::CONFIRM_TIME, time)
            .set(AccountModel::STATUS, AccountStatus::Enable as i8)
            .execute(
                SqlSuffix::Where(&sql_format!("id={}", account.id)),
                &mut *db,
            )
            .await;
        if let Err(ie) = tmp {
            db.rollback().await?;
            return Err(ie.into());
        }
        if let Err(ie) = self
            .index
            .cat_one_add(
                crate::model::AccountIndexCat::AccountStatus,
                account.id,
                &(AccountStatus::Enable as i8).to_string(),
                Some(&mut db),
            )
            .await
        {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;

        self.logger
            .add(
                &LogAccount {
                    action: "enable",
                    nickname: &account.nickname,
                    status: AccountStatus::Enable as i8,
                },
                Some(account.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;

        Ok(())
    }
    //删除用户
    pub async fn del(
        &self,
        account: &AccountModel,
        del_name: Option<&str>,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<()> {
        let time = now_time()?;
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        //delete account data
        let del_name_ow = del_name.map(|e| e.to_string());
        let mut update = Update::<AccountModel>::new()
            .set(AccountModel::STATUS, AccountStatus::Delete as i8)
            .set(AccountModel::CHANGE_TIME, time);
        if let Some(ref name) = del_name_ow {
            update = update.set(AccountModel::NICKNAME, name as &str);
        }
        let tmp = update
            .execute(
                SqlSuffix::Where(&sql_format!("id={}", account.id)),
                &mut *db,
            )
            .await;
        if let Err(e) = tmp {
            db.rollback().await?;
            return Err(e.into());
        }
        if let Err(ie) = self.index.account_del(account.id, Some(&mut db)).await {
            db.rollback().await?;
            return Err(ie);
        }
        db.commit().await?;
        self.cache.clear(&account.id).await;
        self.logger
            .add(
                &LogAccount {
                    action: "del",
                    nickname: &account.nickname,
                    status: AccountStatus::Delete as i8,
                },
                Some(account.id),
                Some(op_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    pub async fn set_nikename(
        &self,
        account: &AccountModel,
        nikename: &str,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<u64> {
        self.nickname_param_valid(nikename).await?;
        let nikename = nikename.to_string();
        let time = now_time().unwrap_or_default();
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::<AccountModel>::new()
            .set(AccountModel::CHANGE_TIME, time)
            .set(AccountModel::NICKNAME, &nikename)
            .execute(
                SqlSuffix::Where(&sql_format!("id={}", account.id)),
                &mut *db,
            )
            .await;
        let out = match res {
            Err(e) => {
                db.rollback().await?;
                Err(e)?
            }
            Ok(mr) => {
                if let Err(ie) = self
                    .index
                    .cat_one_add(
                        crate::model::AccountIndexCat::NikeName,
                        account.id,
                        &account.nickname,
                        Some(&mut db),
                    )
                    .await
                {
                    db.rollback().await?;
                    return Err(ie);
                }

                db.commit().await?;
                self.cache.clear(&account.id).await;
                Ok(mr.last_insert_id())
            }
        };

        self.logger
            .add(
                &LogAccount {
                    action: "nikename",
                    nickname: &account.nickname,
                    status: account.status,
                },
                Some(account.id),
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
        AccountModel,
        AccountResult<AccountModel>,
        id,
        "id = {id} and status in ({status})",
        status = [AccountStatus::Enable as i8, AccountStatus::Init as i8]
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        AccountModel,
        AccountResult<HashMap<u64, AccountModel>>,
        id,
        ids,
        "id in ({ids}) and status in ({status})",
        status = [AccountStatus::Enable as i8, AccountStatus::Init as i8]
    );
    //搜索用户
    pub async fn search(
        &self,
        key_word: &str,
        enable_account: bool,
        limit: &CursorPageParam<u64>,
    ) -> AccountResult<(Vec<AccountItem>, CursorPageData<u64>)> {
        self.index
            .search(
                if enable_account {
                    &[AccountStatus::Enable]
                } else {
                    &[AccountStatus::Enable, AccountStatus::Init]
                },
                key_word,
                &[
                    AccountIndexCat::NikeName,
                    AccountIndexCat::AccountName,
                    AccountIndexCat::Email,
                    AccountIndexCat::Mobile,
                ],
                limit,
            )
            .await
    }

    pub fn cache(&'_ self) -> AccountCache<'_> {
        AccountCache { dao: self }
    }
}

pub struct AccountCache<'t> {
    pub dao: &'t Account,
}
impl AccountCache<'_> {
    lsys_core::impl_cache_fetch_one!(find_by_id, dao, cache, u64, AccountResult<AccountModel>);
    lsys_core::impl_cache_fetch_vec!(
        find_by_ids,
        dao,
        cache,
        u64,
        AccountResult<HashMap<u64, AccountModel>>
    );
}
