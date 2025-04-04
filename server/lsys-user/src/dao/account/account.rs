use std::collections::HashMap;

use std::sync::Arc;

use crate::dao::AccountResult;
use crate::model::{AccountIndexCat, AccountModel, AccountModelRef, AccountStatus};
use lsys_access::dao::{AccessDao, UserInfo};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{fluent_message, now_time, LimitParam, RemoteNotify, RequestEnv};
use lsys_logger::dao::ChangeLoggerDao;
use tracing::warn;

use super::logger::LogAccount;
use super::AccountError;
use super::{AccountIndex, AccountItem};
use lsys_core::db::{Insert, Update};
use lsys_core::db::{ModelTableName, SqlQuote};
use lsys_core::IntoFluentMessage;
use lsys_core::{model_option_set, sql_format};
use sqlx::{Acquire, MySql, Pool, Transaction};
pub struct Account {
    db: Pool<MySql>,
    access: Arc<AccessDao>,
    index: Arc<AccountIndex>,
    pub(crate) cache: Arc<LocalCache<u64, AccountModel>>,
    logger: Arc<ChangeLoggerDao>,
}

// find_by_id_impl!(Account,AccountModel,cache,id,"");

impl Account {
    pub fn new(
        db: Pool<MySql>,
        access: Arc<AccessDao>,
        index: Arc<AccountIndex>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::new(LocalCache::new(remote_notify, config)),
            db,
            access,
            index,
            logger,
        }
    }
    /// 添加用户
    pub async fn add(
        &self,
        nickname: &str,
        op_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> AccountResult<AccountModel> {
        let time = now_time()?;
        let u_status = AccountStatus::Init as i8;
        let nickname_ow = nickname.to_string();
        let new_data = model_option_set!(AccountModelRef,{
            nickname:nickname_ow,
            add_time:time,
            change_time:time,
            use_name:0,
            status:u_status,
        });

        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Insert::<AccountModel, _>::new(new_data)
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

        //此过程必须,同步过去好查数据
        if let Err(err) = self
            .access
            .user
            .sync_user(0, account.id, Some(nickname), None)
            .await
        {
            warn!(
                "sync user to access fail:{}",
                err.to_fluent_message().default_format()
            );
        };

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
        let change = lsys_core::model_option_set!(AccountModelRef,{
            change_time:time,
            confirm_time:time,
            status:AccountStatus::Enable as i8,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let tmp = Update::< AccountModel, _>::new(change)
            .execute_by_pk(account, &mut *db)
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
        let mut change = lsys_core::model_option_set!(AccountModelRef,{
            status:AccountStatus::Delete as i8,
            change_time:time
        });
        let del_name_ow = del_name.map(|e| e.to_string());
        change.nickname = del_name_ow.as_ref();
        let tmp = Update::< AccountModel, _>::new(change)
            .execute_by_pk(account, &mut *db)
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
        let nikename = nikename.trim().to_string();
        if nikename.is_empty() || nikename.len() > 32 {
            return Err(AccountError::System(
                fluent_message!("account-nikename-wrong",{
                        "len":nikename.len(),
                        "max":32
                    }
                    // "accountname length need 1-32 char"
                ),
            ));
        }
        let time = now_time().unwrap_or_default();
        let change = lsys_core::model_option_set!(AccountModelRef,{
            change_time:time,
            nickname:nikename,
        });
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };
        let res = Update::< AccountModel, _>::new(change)
            .execute_by_pk(account, &mut *db)
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

        //此过程必须,通过过去好查数据
        if let Err(err) = self
            .access
            .user
            .sync_user(0, account.id, Some(&account.nickname), None)
            .await
        {
            warn!(
                "sync user nikename to access fail:{}",
                err.to_fluent_message().default_format()
            );
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
        limit: Option<&LimitParam>,
    ) -> AccountResult<(Vec<AccountItem>, Option<u64>)> {
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
    pub async fn get_user(&self, account: &AccountModel) -> AccountResult<UserInfo> {
        Ok(self
            .dao
            .access
            .user
            .cache()
            .sync_user(0, account.id, Some(&account.nickname), None)
            .await?)
    }
}
